-- Add a small synopsis composed of the table name and article ID
UPDATE wrote SET
    metadata.description = record::tb(out) + ' written by ' + <string>in;
