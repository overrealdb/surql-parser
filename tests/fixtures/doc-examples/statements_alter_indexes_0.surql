-- 1. Decommission the index
ALTER INDEX my_index ON my_table PREPARE REMOVE;

-- 2. Monitor query performance and verify queries still work
SELECT ... FROM my_table EXPLAIN;

-- 3. If satisfied, permanently remove the index
REMOVE INDEX my_index ON my_table;
