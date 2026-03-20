; SurrealQL formatting queries for tree-sitter
; Used by the LSP formatter for comment-preserving formatting.
;
; Capture names follow Topiary conventions:
; @append_space / @prepend_space — horizontal spacing
; @append_hardline / @prepend_hardline — vertical spacing
; @indent.start / @indent.end — indentation blocks
; @leaf — preserve content as-is (strings, comments, numbers)

; ── Preserve literals and comments ──
(comment) @leaf
(string) @leaf
(number) @leaf

; ── Keywords: uppercase ──
; (handled by keyword_format in Rust, not here)

; ── Statement separation ──
";" @append_hardline

; ── Block indentation ──
"{" @append_hardline @indent.start
"}" @prepend_hardline @indent.end

; ── Comma-separated lists ──
"," @append_space

; ── Parentheses ──
"(" @indent.start
")" @indent.end
