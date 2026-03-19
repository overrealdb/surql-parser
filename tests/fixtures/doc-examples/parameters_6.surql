/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[1, 2, 3, 4]"

[[test.results]]
value = "[[1, 2], [3, 4]]"

*/

LET $nums = [
    [1,2],
    [3,4]
];

{
    LET $nums = $nums.flatten();
    -- Flattened into a single array,
    -- so $nums is shadowed as [1,2,3,4]
    RETURN $nums;
};

-- Returns original unflattened $nums:
-- [[1,2], [3,4]]
RETURN $nums;
