LET $init = [];

FOR $num IN 1..=3 {
	$init += $num;
};
-- Error: 'assignment operators are only allowed in SET and DUPLICATE KEY UPDATE clauses'

RETURN $init;
