DEFINE INDEX idx_mtree_embedding ON Document FIELDS items.embedding HNSW DIMENSION 4 TYPE I64;
