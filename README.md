# vim-specflow

A Vim plugin that provides syntax highlighting and binding navigation for SpecFlow (Gherkin) feature files.

## Features

- **Syntax Highlighting**: Full syntax highlighting for SpecFlow feature files including:
  - Keywords (Feature, Scenario, Given, When, Then, And, But)
  - Tags (@tag)
  - Comments
  - Tables
  - String parameters
  - Scenario outlines and examples

- **Binding Navigation**: Jump from feature file steps to their corresponding C# step definitions
  - Supports Given, When, Then, And, But steps
  - Searches through C# files for matching binding attributes
  - Pattern matching for parameterized steps

## Installation

### Using Vim 8+ native package management

1. Create the directory structure:
   ```bash
   mkdir -p ~/.vim/pack/plugins/start/
   ```

2. Clone or copy this plugin:
   ```bash
   cd ~/.vim/pack/plugins/start/
   git clone <repository-url> vim-specflow
   ```

### Using Pathogen

```bash
cd ~/.vim/bundle
git clone <repository-url> vim-specflow
```

### Using Vundle

Add to your `.vimrc`:
```vim
Plugin 'vim-specflow'
```

## Usage

### Commands

- `:SpecFlowJumpToBinding` - Jump to the C# binding for the step on the current line
- `:SpecFlowListBindings` - List all SpecFlow bindings found in C# files

### Key Mappings

The following mappings are available in `.feature` files:

- `<Leader>sb` - Jump to step binding
- `<Leader>sl` - List all bindings

### Configuration

You can configure the search paths for C# files:

```vim
let g:specflow_search_paths = ['.', '../src', '../tests']
```

## File Structure

```
vim-specflow/
├── ftdetect/specflow.vim    # File type detection
├── syntax/specflow.vim      # Syntax highlighting rules
├── ftplugin/specflow.vim    # File type specific settings
├── plugin/specflow.vim      # Main plugin functionality
└── README.md               # This file
```

## Syntax Highlighting

The plugin provides comprehensive syntax highlighting for:

- **Keywords**: `Feature`, `Scenario`, `Scenario Outline`, `Background`, `Given`, `When`, `Then`, `And`, `But`, `Examples`
- **Tags**: `@tag_name` 
- **Comments**: Lines starting with `#`
- **Strings**: Quoted text in single or double quotes
- **Parameters**: Text within `<angle_brackets>` or quotes
- **Tables**: Pipe-separated tables with proper column highlighting
- **Feature/Scenario titles**: Special highlighting for section headers

## Binding Navigation

The binding navigation feature helps you quickly jump from feature file steps to their corresponding C# implementations:

1. Place your cursor on a step line (Given, When, Then, And, But)
2. Use `<Leader>sb` or `:SpecFlowJumpToBinding`
3. The plugin will:
   - Extract the step text
   - Convert it to a search pattern
   - Search through C# files for matching `[Given]`, `[When]`, or `[Then]` attributes
   - Jump to the first matching binding

### How it works

The plugin looks for C# binding attributes in this format:
```csharp
[Given(@"the '(.*)' '(machine|domain)' group exists")]
public void TheGroupExists(string groupName, ContextType groupContext)
{
    // implementation
}
```

It converts feature file steps like:
```gherkin
Given the 'Administrators' 'machine' group exists
```

Into search patterns that match the corresponding binding attributes.

## Limitations

- The regex pattern matching is basic and may not handle all complex SpecFlow patterns
- Only searches for standard SpecFlow attributes (`[Given]`, `[When]`, `[Then]`)
- Requires C# files to be in the configured search paths

## Contributing

Feel free to submit issues and pull requests to improve the plugin.

## License

This plugin is released under the MIT License.