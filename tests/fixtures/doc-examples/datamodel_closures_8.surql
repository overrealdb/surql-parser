LET $okay_nums = [1,2,3];

-- Returns [] because $okay_nums not present inside the closure
[1,5,6,7,0].filter(|$n| $n IN $okay_nums);
