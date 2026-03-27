-- Test: create an agent and verify it persists.
CREATE agent:test SET name = 'Alice', role = 'developer', active = true;

LET $result = SELECT * FROM agent:test;
IF $result[0].name != 'Alice' { THROW 'Expected agent name to be Alice'; };
IF $result[0].role != 'developer' { THROW 'Expected agent role to be developer'; };
IF $result[0].active != true { THROW 'Expected agent to be active'; };
