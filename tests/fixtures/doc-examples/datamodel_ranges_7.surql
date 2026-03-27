/**[test]

[[test.results]]
value = "[{ original: 130, square_root: 11.40175425099138f }, { original: 140, square_root: 11.832159566199232f }]"

*/

-- Construct an array
(<array> 1..=100)
-- Turn it into an array that increments by 10
    .map(|$v| $v * 10)
-- Turn each number into a object with original and square root value
    .map(|$v| { original: $v, square_root: math::sqrt($v) })
-- Keep only those with square roots in between 11 and 12
    .filter(|$obj| $obj.square_root IN 11..12);
