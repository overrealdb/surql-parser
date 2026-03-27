CREATE |user:10| VERSION d"2020-09-09";

[[{ id: user:rtbjoqv1xe9wnxjx5aro }, { id: user:tkik878q8uoddvuucu0a }, { id: user:rcnywgogvlipv3tb8qut }, { id: user:30ynx82x52ff77dxzv1i }, { id: user:59mxi0xosi3im5ccbx8l }, { id: user:nolu7yreqs4e5m7255oa }, { id: user:u384ycj1d2esi3yrasli }, { id: user:n4xnrq98ookevhmdd7d2 }, { id: user:5j5ujfu4dokcpdk51qa8 }, { id: user:jiqmlvrgafeorr50nvn9 }]]

CREATE |user:10| VERSION d"2020-09-10";
[[{ id: user:ze98ow4bzdcndzc5nlqj }, { id: user:gjqu2uh3wnp1cpjg1unt }, { id: user:17bxpjl4ptbxv9k2ghmt }, { id: user:fmqqeajf52neg4c7oaoq }, { id: user:bfn45ewsg86auvekeuz0 }, { id: user:834yq1tyatwopb4726mj }, { id: user:veehoua4cu65ff4wc8pf }, { id: user:y3az4pizc0ddpruixw6g }, { id: user:xrn6eqrtyqgg8cgpm9zp }, { id: user:s06acf74rsnvhvim3ys5 }]]

RETURN count(SELECT * FROM user VERSION d"2020-09-09"); -- returns 10
[10]

RETURN count(SELECT * FROM user); -- returns 21
[21]
