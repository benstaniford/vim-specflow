# vim-specflow

A Vim plugin for SpecFlow development with C#.

## What it does

- **Syntax highlighting** for .feature files
- **Jump to C# bindings** with `Ctrl-]`
- **Highlights missing bindings** in red automatically

## Installation

```bash
cd ~/.vim/pack/plugins/start/
git clone <repository-url> vim-specflow
```

## Usage

Open any `.feature` file and:

- Press `Ctrl-]` on a step to jump to its C# binding
- Steps without bindings are highlighted in red
- Use `:SpecFlowListBindings` to see all available bindings

## Configuration

Set the path to your C# files (defaults to searching CWD):

```vim
let g:specflow_search_paths = ['/path/to/your/csharp/project']
```

## Commands

- `:SpecFlowJumpToBinding` - Jump to binding
- `:SpecFlowListBindings` - List all bindings  
- `:SpecFlowClearHighlight` - Clear error highlights
- `:SpecFlowClearCache` - Refresh binding cache
