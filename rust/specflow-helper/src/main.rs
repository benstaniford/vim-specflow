//! CLI for the vim-specflow plugin.
//!
//! Two subcommands, both speaking JSON on stdout:
//!
//!   specflow-helper resolve --root DIR [--kind Given|When|Then] --step TEXT
//!     -> {"file":"...","line":N,"pattern":"..."} on hit
//!     -> {} on miss (exit 0)
//!
//!   specflow-helper scan --root DIR --feature PATH
//!     -> {"feature":"...", "steps":[{"line":N,"kind":"Given","keyword":"Given","text":"...","resolved":{"file":"...","line":N,"pattern":"..."}|null}, ...], "stats":{...}}
//!
//! Common flags: --cache PATH, --no-cache.

use specflow_helper::{
    default_cache_path, parse_feature_steps, Binding, Index, MatchResult, StepKind,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
specflow-helper <subcommand> [args]

Subcommands:
  resolve --root DIR [--kind Given|When|Then] --step TEXT
  scan    --root DIR --feature PATH
  list    --root DIR

Common flags:
  --cache PATH    cache file location (defaults to $XDG_CACHE_HOME/vim-specflow/...)
  --no-cache      build the index from scratch and don't persist
  --version       print version and exit
  --help          print this help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("specflow-helper {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let sub = match args.first() {
        Some(s) => s.as_str(),
        None => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let rest = &args[1..];
    let result = match sub {
        "resolve" => run_resolve(rest),
        "scan" => run_scan(rest),
        "list" => run_list(rest),
        other => Err(format!("unknown subcommand: {other}\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("specflow-helper: {e}");
            ExitCode::from(1)
        }
    }
}

#[derive(Default)]
struct CommonOpts {
    root: Option<PathBuf>,
    cache: Option<PathBuf>,
    no_cache: bool,
}

fn parse_common<'a>(args: &'a [String]) -> Result<(CommonOpts, Vec<&'a str>), String> {
    let mut opts = CommonOpts::default();
    let mut leftover = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--root" => {
                opts.root = Some(PathBuf::from(arg_value(args, i)?));
                i += 2;
            }
            "--cache" => {
                opts.cache = Some(PathBuf::from(arg_value(args, i)?));
                i += 2;
            }
            "--no-cache" => {
                opts.no_cache = true;
                i += 1;
            }
            _ => {
                leftover.push(a);
                i += 1;
            }
        }
    }
    Ok((opts, leftover))
}

fn arg_value(args: &[String], i: usize) -> Result<String, String> {
    args.get(i + 1)
        .cloned()
        .ok_or_else(|| format!("flag {:?} expects a value", args[i]))
}

fn build_index(opts: &CommonOpts) -> Result<Index, String> {
    let root = opts
        .root
        .as_deref()
        .ok_or_else(|| "--root is required".to_string())?;
    let cache_path: Option<PathBuf> = if opts.no_cache {
        None
    } else {
        Some(
            opts.cache
                .clone()
                .unwrap_or_else(|| default_cache_path(root)),
        )
    };
    Index::build(root, cache_path.as_deref()).map_err(|e| format!("index build failed: {e}"))
}

// ---- resolve --------------------------------------------------------------

fn run_resolve(args: &[String]) -> Result<(), String> {
    let (common, rest) = parse_common(args)?;
    let mut kind: Option<StepKind> = None;
    let mut step: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--kind" => {
                let v = rest.get(i + 1).ok_or("--kind expects a value")?;
                kind = parse_kind(v)?;
                i += 2;
            }
            "--step" => {
                step = Some(
                    rest.get(i + 1)
                        .ok_or("--step expects a value")?
                        .to_string(),
                );
                i += 2;
            }
            other => return Err(format!("unknown flag for resolve: {other}")),
        }
    }
    let step = step.ok_or("--step is required")?;
    let idx = build_index(&common)?;
    let query = idx.into_query();
    match query.find(&step, kind) {
        Some(m) => println!("{}", encode_resolved(&m)),
        None => println!("{{}}"),
    }
    Ok(())
}

fn parse_kind(v: &str) -> Result<Option<StepKind>, String> {
    match v {
        "Given" => Ok(Some(StepKind::Given)),
        "When" => Ok(Some(StepKind::When)),
        "Then" => Ok(Some(StepKind::Then)),
        "And" | "But" | "" | "null" | "any" => Ok(None),
        other => Err(format!("invalid --kind: {other:?}")),
    }
}

// ---- scan -----------------------------------------------------------------

fn run_scan(args: &[String]) -> Result<(), String> {
    let (common, rest) = parse_common(args)?;
    let mut feature: Option<PathBuf> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--feature" => {
                feature = Some(PathBuf::from(
                    rest.get(i + 1).ok_or("--feature expects a value")?,
                ));
                i += 2;
            }
            other => return Err(format!("unknown flag for scan: {other}")),
        }
    }
    let feature = feature.ok_or("--feature is required")?;
    let idx = build_index(&common)?;
    let stats = idx.stats.clone();
    let query = idx.into_query();
    let src = std::fs::read_to_string(&feature)
        .map_err(|e| format!("could not read feature {feature:?}: {e}"))?;
    let steps = parse_feature_steps(&src);

    let mut out = String::from("{\"feature\":");
    push_json_string(&mut out, &feature.to_string_lossy());
    out.push_str(",\"steps\":[");
    for (i, step) in steps.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"line\":");
        out.push_str(&step.line.to_string());
        out.push_str(",\"kind\":");
        match step.kind {
            Some(k) => {
                out.push('"');
                out.push_str(k.as_str());
                out.push('"');
            }
            None => out.push_str("null"),
        }
        out.push_str(",\"keyword\":");
        push_json_string(&mut out, &step.keyword);
        out.push_str(",\"text\":");
        push_json_string(&mut out, &step.text);
        out.push_str(",\"resolved\":");
        match query.find(&step.text, step.kind) {
            Some(m) => out.push_str(&encode_resolved(&m)),
            None => out.push_str("null"),
        }
        out.push('}');
    }
    out.push_str("],\"stats\":{");
    out.push_str(&format!(
        "\"files_scanned\":{},\"files_reparsed\":{},\"files_from_cache\":{},\"bindings\":{}",
        stats.files_scanned, stats.files_reparsed, stats.files_from_cache, stats.bindings_total,
    ));
    out.push_str("}}");
    println!("{out}");
    Ok(())
}

// ---- list -----------------------------------------------------------------

fn run_list(args: &[String]) -> Result<(), String> {
    let (common, rest) = parse_common(args)?;
    if !rest.is_empty() {
        return Err(format!("unknown flag for list: {}", rest[0]));
    }
    let idx = build_index(&common)?;
    // Tab-separated: [Kind] pattern \t file:line
    // The format is fzf-friendly — fzf can be told to display only the first
    // field while keeping the location for the sink callback.
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    use std::io::Write;
    for b in idx.bindings.iter() {
        let _ = writeln!(
            handle,
            "[{}] {}\t{}:{}",
            b.kind.as_str(),
            b.pattern,
            b.file.display(),
            b.line
        );
    }
    Ok(())
}

// ---- shared JSON helpers --------------------------------------------------

fn encode_resolved(m: &MatchResult<'_>) -> String {
    let mut s = String::from("{\"file\":");
    push_json_string(&mut s, &m.binding.file.to_string_lossy());
    s.push_str(",\"line\":");
    s.push_str(&m.binding.line.to_string());
    s.push_str(",\"kind\":\"");
    s.push_str(m.binding.kind.as_str());
    s.push_str("\",\"pattern\":");
    push_json_string(&mut s, &m.binding.pattern);
    s.push('}');
    s
}

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

// Silence "unused" complaint when Binding's accessors aren't exercised directly.
#[allow(dead_code)]
fn _ensure_binding_link(_b: &Binding, _p: &Path) {}
