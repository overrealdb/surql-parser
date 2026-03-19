ns/db> CREATE person;
[[{ id: person:epgvviec00l4pnmhtt5n }]]

ns/db> DEFINE DB strict_db STRICT;
[NONE]

ns/db> USE DB strict_db;
[NONE]

ns/strict_db> CREATE person;
["Thrown error: The table 'person' does not exist"]

ns/strict_db> DEFINE TABLE person;
[NONE]

ns/strict_db> CREATE person;
[[{ id: person:c76lfw6n4yb1z2dj9xaj }]]
