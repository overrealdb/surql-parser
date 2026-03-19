/**[test]

[[test.results]]
value = "NONE"

*/

-- Define a relation table, and constrain the type of relation which can be stored
DEFINE TABLE assigned_to SCHEMAFULL TYPE RELATION IN tag OUT sticky
    PERMISSIONS
        FOR create, select, update, delete
            WHERE in.owner == $auth.id AND out.author == $auth.id;
