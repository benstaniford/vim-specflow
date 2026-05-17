" SpecFlow Vim plugin — VimScript shim over the specflow-helper binary.
" The binary handles all indexing, regex matching, and cache management.

if exists('g:loaded_specflow_plugin')
    finish
endif
let g:loaded_specflow_plugin = 1

" --- Configuration -------------------------------------------------------

" Single root directory containing the C# binding sources. Defaults to the
" first entry of the legacy g:specflow_search_paths list for back-compat.
if !exists('g:specflow_root')
    if exists('g:specflow_search_paths') && type(g:specflow_search_paths) == v:t_list && !empty(g:specflow_search_paths)
        let g:specflow_root = g:specflow_search_paths[0]
    else
        let g:specflow_root = ''
    endif
endif

let g:specflow_highlight_unbound = get(g:, 'specflow_highlight_unbound', 1)

" Helper binary location. Defaults to <plugin>/bin/specflow-helper(.exe);
" rebuild via <plugin>/bin/build_helper.sh (Linux/Mac) or build_helper.ps1
" (Windows).
let s:plugin_dir = expand('<sfile>:p:h:h')
let s:exe_suffix = has('win32') ? '.exe' : ''
let g:specflow_helper_path = get(g:, 'specflow_helper_path', s:plugin_dir . '/bin/specflow-helper' . s:exe_suffix)

highlight default SpecFlowUnboundStep ctermfg=White ctermbg=Red guifg=#ffffff guibg=#ff0000

" --- Helper invocation ---------------------------------------------------

function! s:HelperReady() abort
    return filereadable(g:specflow_helper_path) && executable(g:specflow_helper_path)
endfunction

function! s:WarnMissingHelper() abort
    let l:script = has('win32') ? 'build_helper.ps1' : 'build_helper.sh'
    echohl WarningMsg
    echom 'specflow: helper not found at ' . g:specflow_helper_path
    echom '          run ' . s:plugin_dir . '/bin/' . l:script
    echohl None
endfunction

function! s:RunHelper(args) abort
    let l:parts = [g:specflow_helper_path] + a:args
    let l:cmd = join(map(copy(l:parts), 'shellescape(v:val)'), ' ')
    let l:out = system(l:cmd)
    if v:shell_error != 0
        return {'_error': substitute(l:out, '\n\+$', '', '')}
    endif
    try
        return json_decode(l:out)
    catch
        return {'_error': 'invalid JSON from helper: ' . l:out}
    endtry
endfunction

function! s:Root() abort
    return !empty(g:specflow_root) ? g:specflow_root : getcwd()
endfunction

" --- Step extraction -----------------------------------------------------

function! s:ExtractStep(linenr) abort
    let l:trimmed = substitute(getline(a:linenr), '^\s*', '', '')
    for l:kw in ['Given', 'When', 'Then', 'And', 'But']
        if l:trimmed =~# '^' . l:kw . '\s'
            let l:text = substitute(l:trimmed[len(l:kw):], '^\s*', '', '')
            return [l:kw, substitute(l:text, '\s\+$', '', '')]
        endif
    endfor
    return ['', '']
endfunction

" For an And/But step, walk backwards through the current buffer to find the
" preceding concrete step kind. Scenario/Background/Feature/Rule openers
" reset inheritance.
function! s:ResolveInheritedKind(linenr) abort
    let l:i = a:linenr - 1
    while l:i > 0
        let l:t = substitute(getline(l:i), '^\s*', '', '')
        if l:t =~# '^\(Scenario\|Background\|Examples\|Rule\|Feature\):'
            return 'And'
        endif
        for l:kw in ['Given', 'When', 'Then']
            if l:t =~# '^' . l:kw . '\s'
                return l:kw
            endif
        endfor
        let l:i -= 1
    endwhile
    return 'And'
endfunction

" --- Public commands -----------------------------------------------------

function! SpecFlowJumpToBinding() abort
    if !s:HelperReady()
        call s:WarnMissingHelper()
        return
    endif
    let l:linenr = line('.')
    let [l:kind, l:step] = s:ExtractStep(l:linenr)
    if empty(l:step)
        echo 'specflow: no step on current line'
        return
    endif
    if l:kind ==# 'And' || l:kind ==# 'But'
        let l:kind = s:ResolveInheritedKind(l:linenr)
    endif
    let l:result = s:RunHelper(['resolve',
                \ '--root', s:Root(),
                \ '--kind', l:kind,
                \ '--step', l:step])
    if has_key(l:result, '_error')
        echohl WarningMsg | echom 'specflow: ' . l:result._error | echohl None
        return
    endif
    if !has_key(l:result, 'file')
        echo 'specflow: no binding for: ' . l:step
        return
    endif
    execute 'edit ' . fnameescape(l:result.file)
    call cursor(l:result.line, 1)
    normal! zz
endfunction

function! s:HighlightUnbound() abort
    if !g:specflow_highlight_unbound
        return
    endif
    if &filetype !=# 'specflow' && &filetype !=# 'cucumber'
        return
    endif
    if !s:HelperReady()
        if !get(s:, 'warned_missing', 0)
            call s:WarnMissingHelper()
            let s:warned_missing = 1
        endif
        return
    endif
    silent! call clearmatches()
    let l:file = expand('%:p')
    if empty(l:file) || !filereadable(l:file)
        return
    endif
    let l:result = s:RunHelper(['scan',
                \ '--root', s:Root(),
                \ '--feature', l:file])
    if has_key(l:result, '_error') || !has_key(l:result, 'steps')
        return
    endif
    for l:step in l:result.steps
        if type(l:step.resolved) != v:t_dict
            call matchadd('SpecFlowUnboundStep', '\%' . l:step.line . 'l.*')
        endif
    endfor
endfunction

function! s:ClearCache() abort
    let l:base = !empty($XDG_CACHE_HOME) ? $XDG_CACHE_HOME : $HOME . '/.cache'
    let l:dir = l:base . '/vim-specflow'
    if isdirectory(l:dir)
        call delete(l:dir, 'rf')
    endif
    echo 'specflow: cache cleared'
endfunction

function! s:RefreshOpenFeatureBuffers() abort
    let l:cur = bufnr('%')
    for l:b in range(1, bufnr('$'))
        if bufexists(l:b) && bufloaded(l:b)
                    \ && getbufvar(l:b, '&filetype') =~# '^\(specflow\|cucumber\)$'
            execute 'buffer ' . l:b
            call s:HighlightUnbound()
        endif
    endfor
    if bufexists(l:cur)
        execute 'buffer ' . l:cur
    endif
endfunction

command! SpecFlowJumpToBinding call SpecFlowJumpToBinding()
command! SpecFlowHighlightUnbound call s:HighlightUnbound()
command! SpecFlowClearCache call s:ClearCache()
command! SpecFlowClearHighlight call clearmatches()

" .feature files are commonly detected as either 'specflow' (our ftdetect)
" or 'cucumber' (Neovim's built-in). Bind <C-]> on both so the user lands
" on the right side regardless of which detector won.
function! s:SetupBufferMappings() abort
    nnoremap <buffer> <silent> <C-]> :SpecFlowJumpToBinding<CR>
endfunction

augroup SpecFlowPlugin
    autocmd!
    autocmd FileType specflow,cucumber call s:SetupBufferMappings()
    autocmd FileType specflow,cucumber call s:HighlightUnbound()
    autocmd BufRead,BufNewFile *.feature call s:HighlightUnbound()
    autocmd BufWritePost *.feature call s:HighlightUnbound()
    " A .cs save can change binding state; refresh visible feature buffers.
    autocmd BufWritePost *.cs call s:RefreshOpenFeatureBuffers()
augroup END
