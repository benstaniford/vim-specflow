# CLAUDE.md

Notes for Claude when working on this plugin.

## Build the helper

```bash
bin/build_helper.sh       # Linux / Mac
bin\build_helper.ps1      # Windows (PowerShell)
```

Both compile `rust/specflow-helper` in release mode and copy the binary
into `bin/`.

## Run the tests

```bash
cd rust/specflow-helper
cargo test                                              # default suite
SPECFLOW_CORPUS_PATH=/path/to/specflow/tree \
    cargo test --release --test corpus_snapshot_tests -- --ignored
```

Default suite is hermetic. The opt-in corpus snapshot test requires a
real SpecFlow tree at `$SPECFLOW_CORPUS_PATH` and asserts the unbound
rate stays under 1%.

Useful diagnostic examples under `rust/specflow-helper/examples/`:

```bash
cargo run --release --example corpus_smoke   -- <root>   # cold/warm benchmark
cargo run --release --example corpus_unbound -- <root>   # per-file unbound listing
```

## Commit and push

When the user confirms a unit of work is done, commit the changes **and
push to `origin/main`**. The user expects work to land on the remote;
don't leave commits piling up locally.

If the helper binary or `doc/tags` ever stops landing in commits, check
`.gitignore` — the user's global gitignore strips `tags` (matched by an
unignore rule here) and the plugin's own `.gitignore` strips the built
binaries.
