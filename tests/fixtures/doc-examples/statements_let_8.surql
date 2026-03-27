-- Use .map() to turn each string into a bool
LET $vals: array<bool> = some:record.vals.map(|$val| <bool>$val);
LET $len: number = $vals.filter(|$val| $val = true).len();
$len;
