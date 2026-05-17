" Vim syntax file
" Language: SpecFlow (Gherkin) Feature Files
" Maintainer: Claude Code
" Latest Revision: 2025

if exists("b:current_syntax")
  finish
endif

" Keywords
syn keyword specflowKeyword Feature Scenario ScenarioOutline Background Rule Given When Then And But Examples
syn keyword specflowTag contained @[A-Za-z0-9_-]\+ nextgroup=specflowTag skipwhite
syn match specflowTagLine "^\s*@.*$" contains=specflowTag

" Comments  
syn match specflowComment "^\s*#.*$"

" Strings
syn region specflowString start='"' end='"' contained
syn region specflowString start="'" end="'" contained

" Step definitions
syn match specflowGiven "^\s*Given\>" nextgroup=specflowStepText skipwhite
syn match specflowWhen "^\s*When\>" nextgroup=specflowStepText skipwhite  
syn match specflowThen "^\s*Then\>" nextgroup=specflowStepText skipwhite
syn match specflowAnd "^\s*And\>" nextgroup=specflowStepText skipwhite
syn match specflowBut "^\s*But\>" nextgroup=specflowStepText skipwhite

syn match specflowStepText ".*$" contained contains=specflowString,specflowParameter

" Parameters in step definitions
syn match specflowParameter "'[^']*'" contained
syn match specflowParameter '"[^"]*"' contained
syn match specflowParameter "<[^>]*>" contained

" Table headers and rows
syn match specflowTableHeader "^\s*|.*|$" contains=specflowTableSeparator
syn match specflowTableRow "^\s*|.*|$" contains=specflowTableSeparator
syn match specflowTableSeparator "|" contained

" Feature title
syn match specflowFeatureTitle "^\s*Feature:.*$" contains=specflowKeyword

" Scenario titles
syn match specflowScenarioTitle "^\s*Scenario:.*$" contains=specflowKeyword
syn match specflowScenarioOutlineTitle "^\s*Scenario Outline:.*$" contains=specflowKeyword

" Background
syn match specflowBackgroundTitle "^\s*Background:.*$" contains=specflowKeyword

" Examples
syn match specflowExamplesTitle "^\s*Examples:.*$" contains=specflowKeyword

" Highlighting
hi def link specflowKeyword Keyword
hi def link specflowFeatureTitle Title
hi def link specflowScenarioTitle Function
hi def link specflowScenarioOutlineTitle Function
hi def link specflowBackgroundTitle Function
hi def link specflowExamplesTitle Function
hi def link specflowGiven Statement
hi def link specflowWhen Conditional
hi def link specflowThen Type
hi def link specflowAnd Special
hi def link specflowBut Special
hi def link specflowStepText Normal
hi def link specflowParameter String
hi def link specflowString String
hi def link specflowTableHeader PreProc
hi def link specflowTableRow Normal
hi def link specflowTableSeparator Special
hi def link specflowComment Comment
hi def link specflowTag Tag
hi def link specflowTagLine Tag

let b:current_syntax = "specflow"