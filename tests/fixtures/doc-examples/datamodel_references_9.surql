/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ comments: [], id: person:one }]"

[[test.results]]
value = "[{ author: person:one, id: comment:4u9gw4bl2q63zuos3v1y, text: '5/10 for this blog post. The problems I have with it are...' }]"

[[test.results]]
value = "[{ author: person:one, id: comment:em2543ghwy9k921eob3d, text: 'WOW! I never knew you could cut a rope with an arrow.' }]"

[[test.results]]
value = "[{ author: person:one, id: comment:4u9gw4bl2q63zuos3v1y, text: '5/10 for this blog post. The problems I have with it are...' }, { author: person:one, id: comment:em2543ghwy9k921eob3d, text: 'WOW! I never knew you could cut a rope with an arrow.' }]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[]"

*/

DEFINE FIELD author ON comment TYPE record<person> REFERENCE ON DELETE CASCADE;
DEFINE FIELD comments ON person COMPUTED <~comment;

CREATE person:one;
CREATE comment SET author = person:one, text = "5/10 for this blog post. The problems I have with it are...";
CREATE comment SET author = person:one, text = "WOW! I never knew you could cut a rope with an arrow.";

-- Show all the details of comments for 'person:one'
person:one.comments.*.*;
DELETE person:one;
-- Comments no longer exist
SELECT * FROM comment;
