# vim-specflow

A Vim/Neovim plugin for working with SpecFlow `.feature` files alongside a
C# binding tree.

## What it does

- Syntax highlighting for `.feature` files
- `Ctrl-]` on a step jumps to the matching `[Given]`/`[When]`/`[Then]`
  attribute in the C# bindings
- Steps with no matching binding are highlighted in red

The indexing and regex matching is done by a small Rust helper
(`specflow-helper`) so opens, jumps, and highlight passes are fast even on
large test suites.

## Installation

Clone into a Vim packpath and build the helper.

Linux/Mac:

```bash
cd ~/.vim/pack/plugins/start/
git clone <repository-url> vim-specflow
vim-specflow/bin/build_helper.sh
```

Windows (PowerShell):

```powershell
cd $env:USERPROFILE\vimfiles\pack\plugins\start
git clone <repository-url> vim-specflow
powershell -ExecutionPolicy Bypass -File vim-specflow\bin\build_helper.ps1
```

Both scripts need `cargo` on `PATH` (install from <https://rustup.rs>).
They compile `rust/specflow-helper` in release mode and copy the binary
(`specflow-helper` on Linux/Mac, `specflow-helper.exe` on Windows) into the
plugin's `bin/` directory.

## Configuration

```vim
" Single root directory containing the C# binding sources.
let g:specflow_root = '/path/to/your/csharp/project'

" Turn off red highlighting of unbound steps (default: on).
let g:specflow_highlight_unbound = 0

" Override the helper binary location (default: <plugin>/bin/specflow-helper).
let g:specflow_helper_path = '/custom/path/specflow-helper'
```

For backward compatibility, `g:specflow_search_paths = [...]` is still
honoured; the first entry is used as the root.

## Commands

- `:SpecFlowJumpToBinding` — jump from the step under the cursor to its
  binding (default mapping: `Ctrl-]`)
- `:SpecFlowHighlightUnbound` — refresh the unbound-step highlights for the
  current buffer
- `:SpecFlowClearHighlight` — clear all match highlights in the buffer
- `:SpecFlowClearCache` — delete the on-disk binding index cache (forces a
  full rescan on the next call)

## How matching works

The helper extracts every `[Given/When/Then(@"...")]` attribute from the
configured root, compiles each pattern as a real Rust regex, and anchors
matches with `^…$` to mirror SpecFlow's semantics. `And`/`But` steps inherit
the kind of the preceding concrete step in their scenario.

Bindings are cached on disk under `$XDG_CACHE_HOME/vim-specflow/`
(falls back to `$HOME/.cache/vim-specflow/`), keyed by file `(mtime, size)`.
A typical 1,000-binding tree scans in under 100 ms even cold.
