/**[test]

[[test.results]]
value = "[{ id: user:uz4nrrdn0xybrceh4bm9, member_of: 'group1', name: 'User1' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: user:07o9ep802fiorrr936u5, member_of: 'group1', name: 'User2' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: user:ij9yjba7n2428yz8nui0, member_of: 'group1', name: 'User3' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ group_members: ['User2', 'User3', 'User1'], name: 'User1' }]"

*/

CREATE user SET name = "User1", member_of = "group1";
CREATE user SET name = "User2", member_of = "group1";
CREATE user SET name = "User3", member_of = "group1";
SELECT name, 
    (SELECT VALUE name FROM user WHERE member_of = $parent.member_of)
    AS group_members
    FROM user
    WHERE name = "User1";
