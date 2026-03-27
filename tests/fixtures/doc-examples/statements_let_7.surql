LET $vals: array<bool> = some:record.vals;
LET $len: number = $vals.filter(|$val| $val = true).len();
$len;
