/**[test]

[[test.results]]
value = "'<h1>Safe Title</h1><p>Safe paragraph. Not safe <span>event</span>.</p>'"

*/

RETURN string::html::sanitize("<h1>Safe Title</h1><script>alert('XSS')</script><p>Safe paragraph. Not safe <span onload='logout()'>event</span>.</p>");

-- '<h1>Safe Title</h1><p>Safe paragraph. Not safe <span>event</span>.</p>'
