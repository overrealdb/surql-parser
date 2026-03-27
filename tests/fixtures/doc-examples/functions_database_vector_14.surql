/**[test]

[[test.results]]
value = "[{ id: pts:1, point: [1, 2, 3, 4] }]"

[[test.results]]
value = "[{ id: pts:2, point: [4, 5, 6, 7] }]"

[[test.results]]
value = "[{ id: pts:3, point: [8, 9, 10, 11] }]"

[[test.results]]
value = "[{ dist: 2f, id: pts:1 }, { dist: 4f, id: pts:2 }]"

*/
CREATE pts:1 SET point = [1,2,3,4];
CREATE pts:2 SET point = [4,5,6,7];
CREATE pts:3 SET point = [8,9,10,11];
SELECT id, vector::distance::knn() AS dist FROM pts WHERE point <|2,EUCLIDEAN|> [2,3,4,5];
