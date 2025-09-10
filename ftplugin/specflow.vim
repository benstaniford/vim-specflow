" SpecFlow filetype plugin
" Additional settings and mappings for SpecFlow files

if exists("b:did_ftplugin")
    finish
endif
let b:did_ftplugin = 1

" Basic settings
setlocal commentstring=#\ %s
setlocal comments=:#

" Indentation
setlocal expandtab
setlocal shiftwidth=4
setlocal tabstop=4
setlocal softtabstop=4

" Folding
setlocal foldmethod=syntax
setlocal foldlevel=1

" Define folding for scenarios
syntax region specflowScenarioFold start="^\s*Scenario\|^\s*Scenario Outline" end="^\s*$\|^\ze\s*Scenario\|^\ze\s*Scenario Outline\|^\ze\s*Feature" fold transparent

let b:undo_ftplugin = "setlocal commentstring< comments< expandtab< shiftwidth< tabstop< softtabstop< foldmethod< foldlevel<"