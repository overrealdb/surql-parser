; SurrealQL highlights for tree-sitter (SurrealDB 3+)
;
; Node names match grammar.js kw() aliases:
;   kw("SCHEMAFULL") → keyword_schemafull
;   kw("CONTAINSALL") → keyword_containsall (no underscore)

; ─── Keywords ───
[
  (keyword_select)
  (keyword_from)
  (keyword_let)
  (keyword_else)
  (keyword_only)
  (keyword_value)
  (keyword_as)
  (keyword_omit)
  (keyword_explain)
  (keyword_full)
  (keyword_parallel)
  (keyword_timeout)
  (keyword_fetch)
  (keyword_limit)
  (keyword_by)
  (keyword_rand)
  (keyword_collate)
  (keyword_numeric)
  (keyword_asc)
  (keyword_desc)
  (keyword_order)
  (keyword_with)
  (keyword_index)
  (keyword_where)
  (keyword_split)
  (keyword_at)
  (keyword_group)
  (keyword_begin)
  (keyword_cancel)
  (keyword_commit)
  (keyword_transaction)
  (keyword_and)
  (keyword_or)
  (keyword_is)
  (keyword_not)
  (keyword_contains)
  (keyword_containsnot)
  (keyword_containsall)
  (keyword_containsany)
  (keyword_containsnone)
  (keyword_inside)
  (keyword_in)
  (keyword_notinside)
  (keyword_allinside)
  (keyword_anyinside)
  (keyword_noneinside)
  (keyword_outside)
  (keyword_intersects)
  (keyword_chebyshev)
  (keyword_cosine)
  (keyword_euclidean)
  (keyword_hamming)
  (keyword_jaccard)
  (keyword_manhattan)
  (keyword_minkowski)
  (keyword_pearson)
  (keyword_define)
  (keyword_analyzer)
  (keyword_event)
  (keyword_field)
  (keyword_function)
  (keyword_namespace)
  (keyword_param)
  (keyword_drop)
  (keyword_schemafull)
  (keyword_schemaless)
  (keyword_live)
  (keyword_diff)
  (keyword_flexible)
  (keyword_readonly)
  (keyword_jwks)
  (keyword_bm25)
  (keyword_doc_ids_cache)
  (keyword_doc_ids_order)
  (keyword_doc_lengths_cache)
  (keyword_doc_lengths_order)
  (keyword_postings_cache)
  (keyword_postings_order)
  (keyword_terms_cache)
  (keyword_terms_order)
  (keyword_highlights)
  (keyword_any)
  (keyword_normal)
  (keyword_relation)
  (keyword_out)
  (keyword_to)
  (keyword_changefeed)
  (keyword_include)
  (keyword_original)
  (keyword_content)
  (keyword_merge)
  (keyword_patch)
  (keyword_before)
  (keyword_after)
  (keyword_table)
  (keyword_root)
  (keyword_use)
  (keyword_ns)
  (keyword_db)
  (keyword_on)
  (keyword_user)
  (keyword_roles)
  (keyword_remove)
  (keyword_create)
  (keyword_delete)
  (keyword_update)
  (keyword_insert)
  (keyword_into)
  (keyword_tokenizers)
  (keyword_filters)
  (keyword_when)
  (keyword_then)
  (keyword_type)
  (keyword_default)
  (keyword_assert)
  (keyword_permissions)
  (keyword_relate)
  (keyword_ignore)
  (keyword_values)
  (keyword_for)
  (keyword_info)
  (keyword_show)
  (keyword_changes)
  (keyword_since)
  (keyword_comment)
  (keyword_fields)
  (keyword_columns)
  (keyword_unique)
  (keyword_search)
  (keyword_session)
  (keyword_signin)
  (keyword_signup)
  (keyword_if)
  (keyword_exists)
  (keyword_database)
  (keyword_password)
  (keyword_passhash)
  (keyword_count)
  (keyword_set)
  (keyword_return)
  (keyword_overwrite)
  (keyword_throw)
  (keyword_unset)
  (keyword_always)
  (keyword_alter)
  (keyword_break)
  (keyword_continue)
  (keyword_sleep)
  (keyword_kill)
  (keyword_mtree)
  (keyword_dimension)
  (keyword_dist)
  (keyword_efc)
  (keyword_m)
  (keyword_capacity)
  (keyword_hnsw)
  (keyword_owner)
  (keyword_editor)
  (keyword_viewer)
  (keyword_duration)
  (keyword_enforced)
  (keyword_algorithm)
  (keyword_key)
  (keyword_url)
  (keyword_jwt)
  (keyword_record)
  (keyword_bearer)
  (keyword_authenticate)
  (keyword_grant)
  (keyword_access)
  (keyword_upsert)
] @keyword

; ─── Operators ───
(operator) @operator

; ─── Literals ───
[
  (string)
  (prefixed_string)
] @string

[
  (int)
  (float)
  (decimal)
] @number

(duration) @number

[
  (keyword_true)
  (keyword_false)
] @boolean

[
  (keyword_none)
  (keyword_null)
] @constant.builtin

; ─── Identifiers ───
(identifier) @variable
(variable_name) @variable.builtin

[
  (custom_function_name)
  (function_name)
] @function

; ─── Comments ───
(comment) @comment

; ─── Punctuation ───
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," "|" ";"] @punctuation.delimiter

; ─── Types ───
(type_name) @type
(parameterized_type) @type

; ─── Table names (highlight identifiers in table position) ───
(define_table (identifier) @type)
(define_field (identifier) @type)
(define_index (identifier) @type)
(define_event (identifier) @type)
(insert_statement (identifier) @type)

; ─── Special nodes ───
(graph_path) @operator
(cast_expression) @type
(record_id (object_key) @type.builtin)

; ─── Properties ───
(field_assignment (field_path (identifier) @property))
(object_property (object_key) @property)
(subscript (identifier) @property)

; ─── Control flow (more specific highlight) ───
[
  (keyword_if)
  (keyword_else)
  (keyword_for)
  (keyword_break)
  (keyword_continue)
] @keyword.control

; ─── Storage ───
[
  (keyword_let)
  (keyword_set)
  (keyword_unset)
] @keyword.storage

; ─── Errors ───
(ERROR) @error
