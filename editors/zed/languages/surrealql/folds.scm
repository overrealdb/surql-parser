; Foldable code blocks
(block) @fold

; DML statements
(select_statement) @fold
(create_statement) @fold
(update_statement) @fold
(delete_statement) @fold
(insert_statement) @fold
(upsert_statement) @fold
(relate_statement) @fold

; DDL — DEFINE statements
(define_table) @fold
(define_field) @fold
(define_index) @fold
(define_function) @fold
(define_event) @fold
(define_access) @fold
(define_analyzer) @fold
(define_namespace) @fold
(define_database) @fold
(define_param) @fold
(define_user) @fold

; Control flow
(if_statement) @fold
(for_statement) @fold

; Nested structures
(sub_query) @fold
(array) @fold
(object) @fold

; Multi-line clauses
(permissions_clause) @fold
(permissions_basic_clause) @fold
(from_clause) @fold
(table_view_clause) @fold
