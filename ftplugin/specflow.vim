" SpecFlow filetype plugin.

if exists("b:did_ftplugin")
    finish
endif
let b:did_ftplugin = 1

setlocal commentstring=#\ %s
setlocal comments=:#
setlocal expandtab
setlocal shiftwidth=4
setlocal tabstop=4
setlocal softtabstop=4
setlocal foldmethod=syntax
setlocal foldlevel=1

syntax region specflowScenarioFold start="^\s*Scenario\|^\s*Scenario Outline" end="^\s*$\|^\ze\s*Scenario\|^\ze\s*Scenario Outline\|^\ze\s*Feature" fold transparent

nnoremap <buffer> <silent> <C-]> :SpecFlowJumpToBinding<CR>

let b:undo_ftplugin = "setlocal commentstring< comments< expandtab< shiftwidth< tabstop< softtabstop< foldmethod< foldlevel< | silent! nunmap <buffer> <C-]>"
