/**[test]

[[test.results]]
value = "[{ id: product:bt4cwqd5cwwmp68w3dps, name: 'Salesforce', url: 'salesforce.com' }]"
skip-record-id-key = true

*/

INSERT INTO product (name, url) VALUES ('Salesforce', 'salesforce.com') ON DUPLICATE KEY UPDATE tags += 'crm';
