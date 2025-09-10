" SpecFlow Vim Plugin
" Provides syntax highlighting and binding navigation for SpecFlow feature files
" Author: Claude Code

if exists('g:loaded_specflow_plugin')
    finish
endif
let g:loaded_specflow_plugin = 1

" Configuration
let g:specflow_search_paths = get(g:, 'specflow_search_paths', ['/home/ben/Code/epm-windows'])
let g:specflow_cache_enabled = get(g:, 'specflow_cache_enabled', 1)
let g:specflow_highlight_unbound = get(g:, 'specflow_highlight_unbound', 1)

" Cache for bindings to avoid repeated filesystem scans
let s:binding_cache = {}
let s:cache_timestamp = 0
let s:cache_valid_seconds = 300  " 5 minutes

" Define highlight group for unbound steps
highlight default SpecFlowUnboundStep ctermfg=White ctermbg=Red guifg=#ffffff guibg=#ff0000

function! s:EscapeRegex(text)
    return escape(a:text, '[](){}^$.*+?|\\')
endfunction

function! s:ExtractStepText(line)
    " Extract the step text from a Gherkin line
    let l:line = substitute(a:line, '^\s*', '', '')  " Remove leading whitespace
    let l:keywords = ['Given', 'When', 'Then', 'And', 'But']
    
    for keyword in l:keywords
        if l:line =~# '^' . keyword . '\s'
            let l:step_text = substitute(l:line, '^' . keyword . '\s*', '', '')
            return l:step_text
        endif
    endfor
    
    return ''
endfunction

function! s:ConvertStepToRegex(step_text)
    " Convert a Gherkin step to a regex pattern that would match C# binding
    let l:pattern = a:step_text
    
    " Replace quoted strings with capture groups - need to preserve the quotes in the pattern
    " This handles the actual text that appears in the step
    let l:pattern = substitute(l:pattern, "'[^']*'", "'(.*)'", 'g')
    let l:pattern = substitute(l:pattern, '"[^"]*"', '"(.*)"', 'g')
    
    " Replace parameter placeholders with capture groups  
    let l:pattern = substitute(l:pattern, '<[^>]*>', '(.*)', 'g')
    
    " Escape special regex characters but preserve our capture groups
    let l:escaped = s:EscapeRegex(l:pattern)
    
    " Convert our escaped capture groups back to proper regex
    let l:escaped = substitute(l:escaped, "\\\\('\\\\(.*\\\\)'\\\\)", "'(.*)'", 'g')
    let l:escaped = substitute(l:escaped, '\\\\("\\\\(.*\\\\)"\\\\)', '"(.*)"', 'g')
    let l:escaped = substitute(l:escaped, '\\\\((.*)\\\\)', '(.*)', 'g')
    
    return l:escaped
endfunction

function! s:FindCSFiles()
    " Find all C# files in the current directory and subdirectories
    let l:cs_files = []
    
    for search_path in g:specflow_search_paths
        let l:files = split(globpath(search_path, '**/*.cs'), '\n')
        let l:cs_files = l:cs_files + l:files
    endfor
    
    return l:cs_files
endfunction

function! s:SearchBindingInFile(file, step_text, step_type)
    " Search for a binding pattern in a specific C# file
    let l:lines = readfile(a:file)
    let l:line_num = 0
    
    for line in l:lines
        let l:line_num += 1
        
        " Check if this line contains a SpecFlow binding attribute
        if line =~# '^\s*\[' . a:step_type . '(@".*")\]'
            let l:attr_pattern = substitute(line, '^\s*\[' . a:step_type . '(@"\(.*\)")\].*$', '\1', '')
            
            " Now test if our step text would match this C# regex pattern
            if s:TestStepAgainstCSharpRegex(a:step_text, l:attr_pattern)
                return [a:file, l:line_num]
            endif
        endif
    endfor
    
    return []
endfunction

function! s:TestStepAgainstCSharpRegex(step_text, csharp_regex)
    " Test if a step text matches a C# regex pattern
    " Simplified approach: treat ALL parameters as wildcards
    
    " Normalize the step text - replace all quoted content and <params> with PARAM
    let l:normalized_step = substitute(a:step_text, '\s\+$', '', '')  " Remove trailing whitespace
    let l:normalized_step = substitute(l:normalized_step, "'[^']*'", 'PARAM', 'g')
    let l:normalized_step = substitute(l:normalized_step, '"[^"]*"', 'PARAM', 'g')  
    let l:normalized_step = substitute(l:normalized_step, '<[^>]*>', 'PARAM', 'g')
    
    " Normalize the C# regex - be more aggressive about replacing patterns
    let l:normalized_regex = substitute(a:csharp_regex, '\s\+$', '', '')  " Remove trailing whitespace
    
    " Handle quoted alternations more specifically
    " Pattern: '(word1|word2|word3)' should match any quoted content
    let l:normalized_regex = substitute(l:normalized_regex, "'([^']*)'", 'PARAM', 'g')
    let l:normalized_regex = substitute(l:normalized_regex, '"([^"]*)"', 'PARAM', 'g')
    
    " Handle optional trailing patterns like ( and ignore the expiry warning|)
    " This should match either the text or nothing (empty alternative)
    let l:normalized_regex = substitute(l:normalized_regex, '(\s*and[^|)]*|)', '', 'g')
    
    " Handle any remaining parentheses groups
    while l:normalized_regex =~# '([^)]*)'
        let l:normalized_regex = substitute(l:normalized_regex, '([^)]*)', 'PARAM', '')
    endwhile
    
    " Simple string comparison
    return l:normalized_step ==# l:normalized_regex
endfunction

function! s:IsCacheValid()
    " Check if the binding cache is still valid
    if !g:specflow_cache_enabled
        return 0
    endif
    
    let l:current_time = localtime()
    return !empty(s:binding_cache) && (l:current_time - s:cache_timestamp) < s:cache_valid_seconds
endfunction

function! s:BuildBindingCache()
    " Build cache of all bindings found in C# files
    if s:IsCacheValid()
        return s:binding_cache
    endif
    
    let s:binding_cache = {}
    let l:cs_files = s:FindCSFiles()
    
    for cs_file in l:cs_files
        let l:lines = readfile(cs_file)
        let l:line_num = 0
        
        for line in l:lines
            let l:line_num += 1
            if line =~# '^\s*\[\(Given\|When\|Then\)(@".*")\]'
                let l:step_type = substitute(line, '^\s*\[\(\w\+\)(@".*")\].*$', '\1', '')
                let l:pattern = substitute(line, '^\s*\[\w\+(@"\(.*\)")\].*$', '\1', '')
                
                " Store binding info
                let l:binding_key = l:step_type . ':' . l:pattern
                let s:binding_cache[l:binding_key] = {
                    \ 'file': cs_file,
                    \ 'line': l:line_num,
                    \ 'type': l:step_type,
                    \ 'pattern': l:pattern
                \ }
            endif
        endfor
    endfor
    
    let s:cache_timestamp = localtime()
    return s:binding_cache
endfunction

function! s:FindBindingForStep(step_text, step_type)
    " Find a binding for the given step using cache
    let l:cache = s:BuildBindingCache()
    
    for [key, binding] in items(l:cache)
        " And/But steps should match any binding type
        let l:should_check = 0
        if a:step_type =~# '\(Given\|When\|Then\)'
            " And/But - check all binding types
            let l:should_check = 1
        elseif binding.type ==# a:step_type
            " Exact type match
            let l:should_check = 1
        endif
        
        if l:should_check && s:TestStepAgainstCSharpRegex(a:step_text, binding.pattern)
            return binding
        endif
    endfor
    
    return {}
endfunction

function! SpecFlowJumpToBinding()
    " Main function to jump to the binding for the current step
    let l:current_line = getline('.')
    let l:step_text = s:ExtractStepText(l:current_line)
    
    if empty(l:step_text)
        echo "No step found on current line"
        return
    endif
    
    " Determine step type
    let l:step_type = ''
    if l:current_line =~# '^\s*Given'
        let l:step_type = 'Given'
    elseif l:current_line =~# '^\s*When'  
        let l:step_type = 'When'
    elseif l:current_line =~# '^\s*Then'
        let l:step_type = 'Then'
    elseif l:current_line =~# '^\s*And'
        " For And/But, we need to look at the previous step to determine type
        " For simplicity, we'll search all types
        let l:step_type = '\(Given\|When\|Then\)'
    elseif l:current_line =~# '^\s*But'
        let l:step_type = '\(Given\|When\|Then\)'
    else
        echo "Unknown step type"
        return
    endif
    
    " Use cached binding search
    let l:binding = s:FindBindingForStep(l:step_text, l:step_type)
    
    if !empty(l:binding)
        execute 'edit ' . fnameescape(l:binding.file)
        execute l:binding.line
        return
    endif
    
    echo "No binding found for: " . l:step_text
endfunction

function! SpecFlowListBindings()
    " List all bindings found in C# files using cache
    let l:cache = s:BuildBindingCache()
    
    if empty(l:cache)
        echo "No SpecFlow bindings found"
        return
    endif
    
    echo "Found " . len(l:cache) . " bindings:"
    for [key, binding] in items(l:cache)
        echo binding.type . ": " . binding.pattern . " (" . binding.file . ":" . binding.line . ")"
    endfor
endfunction

function! s:HighlightUnboundSteps()
    " Highlight steps that don't have corresponding bindings
    if !g:specflow_highlight_unbound || (&filetype !=# 'specflow' && &filetype !=# 'cucumber')
        return
    endif
    
    " Clear previous matches
    silent! call clearmatches()
    
    let l:cache = s:BuildBindingCache()
    let l:line_num = 1
    let l:total_lines = line('$')
    let l:unbound_count = 0
    
    while l:line_num <= l:total_lines
        let l:line = getline(l:line_num)
        let l:step_text = s:ExtractStepText(l:line)
        
        " Also check for lines that look like steps but don't have keywords
        let l:looks_like_step = l:line =~# '^\s*[A-Z][^:]*$' && l:line !~# '^\s*\(@\|#\|Feature:\|Scenario:\|Background:\|Examples:\)'
        
        if !empty(l:step_text) || l:looks_like_step
            " Determine step type
            let l:step_type = ''
            if l:line =~# '^\s*Given'
                let l:step_type = 'Given'
            elseif l:line =~# '^\s*When'  
                let l:step_type = 'When'
            elseif l:line =~# '^\s*Then'
                let l:step_type = 'Then'
            elseif l:line =~# '^\s*And'
                let l:step_type = '\(Given\|When\|Then\)'
            elseif l:line =~# '^\s*But'
                let l:step_type = '\(Given\|When\|Then\)'
            elseif l:looks_like_step
                " Line looks like a step but has no keyword - highlight as error
                call matchadd('SpecFlowUnboundStep', '\%' . l:line_num . 'l.*')
                let l:unbound_count += 1
                let l:line_num += 1
                continue
            endif
            
            if !empty(l:step_type) && !empty(l:step_text)
                let l:binding = s:FindBindingForStep(l:step_text, l:step_type)
                if empty(l:binding)
                    " Highlight unbound step
                    call matchadd('SpecFlowUnboundStep', '\%' . l:line_num . 'l.*')
                    let l:unbound_count += 1
                endif
            endif
        endif
        
        let l:line_num += 1
    endwhile
    
endfunction

function! s:ClearCache()
    " Clear the binding cache
    let s:binding_cache = {}
    let s:cache_timestamp = 0
endfunction





" Commands
command! SpecFlowJumpToBinding call SpecFlowJumpToBinding()
command! SpecFlowListBindings call SpecFlowListBindings()
command! SpecFlowHighlightUnbound call s:HighlightUnboundSteps()
command! SpecFlowClearCache call s:ClearCache()
command! SpecFlowClearHighlight call clearmatches()

" Key mappings and auto-highlighting (only in feature files)
augroup SpecFlowMappings
    autocmd!
    " Try multiple events to ensure the mapping takes
    autocmd FileType specflow call s:SetupSpecFlowMappings()
    autocmd FileType cucumber call s:SetupSpecFlowMappings()
    autocmd BufRead,BufNewFile *.feature call s:SetupSpecFlowMappings()
    
    " Automatic highlighting of unbound steps
    autocmd BufRead,BufNewFile *.feature call s:HighlightUnboundSteps()
    autocmd BufWritePost *.feature call s:HighlightUnboundSteps()
    autocmd FileType cucumber call s:HighlightUnboundSteps()
    
    " Cache invalidation when C# files change
    autocmd BufWritePost *.cs call s:ClearCache()
augroup END

function! s:SetupSpecFlowMappings()
    " Force override the built-in Ctrl-] mapping
    " First try to unmap any existing mapping
    silent! nunmap <buffer> <C-]>
    silent! unmap <buffer> <C-]>
    
    " Now set our mapping
    nnoremap <buffer> <silent> <C-]> :SpecFlowJumpToBinding<CR>
    noremap <buffer> <silent> <C-]> :SpecFlowJumpToBinding<CR>
    nnoremap <buffer> <Leader>sb :SpecFlowJumpToBinding<CR>
    nnoremap <buffer> <Leader>sl :SpecFlowListBindings<CR>
    
endfunction