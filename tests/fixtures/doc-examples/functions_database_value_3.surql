/**[test]

[[test.results]]
value = "[{ op: 'change', path: '/company', value: '@@ -2,8 +2,10 @@
 urrealDB
+!!
' }, { op: 'add', path: '/latest_version', value: '2.0' }, { op: 'add', path: '/location', value: city:london }]"

*/

{ company: 'SurrealDB' }.diff({ company: 'SurrealDB!!', latest_version: '2.0', location: city:london });
