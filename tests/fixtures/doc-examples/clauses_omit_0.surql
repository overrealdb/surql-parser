/**[test]

[[test.results]]
value = "[{ id: person:tobie, name: 'Tobie', opts: { enabled: true, security: 'secure' }, password: '123456' }]"

[[test.results]]
value = "[{ id: person:jaime, name: 'Jaime', opts: { enabled: false, security: 'secure' }, password: 'asdfgh' }]"

[[test.results]]
value = "[{ id: person:jaime, name: 'Jaime', opts: { enabled: false, security: 'secure' }, password: 'asdfgh' }, { id: person:tobie, name: 'Tobie', opts: { enabled: true, security: 'secure' }, password: '123456' }]"

[[test.results]]
value = "[{ id: person:jaime, name: 'Jaime', opts: { enabled: false } }, { id: person:tobie, name: 'Tobie', opts: { enabled: true } }]"

[[test.results]]
value = "[{ id: person:jaime, name: 'Jaime', opts: {  } }, { id: person:tobie, name: 'Tobie', opts: {  } }]"

*/

CREATE person:tobie SET
	name = 'Tobie',
	password = '123456',
	opts.security = 'secure',
	opts.enabled = true;
CREATE person:jaime SET
	name = 'Jaime',
	password = 'asdfgh',
	opts.security = 'secure',
	opts.enabled = false;

SELECT * FROM person;
-- Omit the password field and security field in the options object
SELECT * OMIT password, opts.security FROM person;

-- Using destructuring syntax (since 2.0.0)
SELECT * OMIT password, opts.{ security, enabled } FROM person;
