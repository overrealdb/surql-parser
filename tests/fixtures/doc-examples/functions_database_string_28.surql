/**[test]

[[test.results]]
value = "'&lt;h1&gt;Safe&#32;Title&lt;&#47;h1&gt;&lt;script&gt;alert(&apos;XSS&apos;)&lt;&#47;script&gt;&lt;p&gt;Safe&#32;paragraph.&#32;Not&#32;safe&#32;&lt;span&#32;onload&#61;&apos;logout()&apos;&gt;event&lt;&#47;span&gt;.&lt;&#47;p&gt;'"

*/

RETURN string::html::encode("<h1>Safe Title</h1><script>alert('XSS')</script><p>Safe paragraph. Not safe <span onload='logout()'>event</span>.</p>");

-- '&lt;h1&gt;Safe&#32;Title&lt;&#47;h1&gt;&lt;script&gt;alert(&apos;XSS&apos;)&lt;&#47;script&gt;&lt;p&gt;Safe&#32;paragraph.&#32;Not&#32;safe&#32;&lt;span&#32;onload&#61;&apos;logout()&apos;&gt;event&lt;&#47;span&gt;.&lt;&#47;p&gt;'
