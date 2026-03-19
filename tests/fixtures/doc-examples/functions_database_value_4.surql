/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ company: 'SurrealDB', latest_version: '3.0' }"

*/


LET $company = {
    company: 'SurrealDB',
    latest_version: '1.5.4'
};

$company.patch([{
		'op': 'replace',
		'path': 'latest_version',
		'value': '3.0'
}]);
