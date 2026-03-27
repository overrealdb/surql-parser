CREATE city:one, city:two, city:three;
RELATE city:one -> to -> city:two SET distance = 25.5;
RELATE city:two -> to -> city:three SET distance = 4.1;
[
	city:one,
	city:two,
	city:three
].map(|$v| { {
	city: $v,
	distance: 0,
	from: NONE,
	to: NONE,
	trips: 0
} }).fold({
	city: NONE,
	distance: 0,
	from: NONE,
	to: NONE,
	trips: 0
}, |$acc, $val, $i| {
	RETURN IF $i = 0 {
		{
			city: $val.city,
			distance: 0,
			from: $val.city,
			to: NONE,
			trips: $acc.trips + 1
		}
  }
	ELSE {
		{
			city: $val.city,
			distance: (SELECT VALUE distance FROM ONLY to WHERE in = $acc.city AND out = $val.city LIMIT 1) + $acc.distance,
			from: $acc.from,
			to: $val.city,
			trips: $acc.trips + 1
		}
  };
}).chain(|$v| { {
	distance: $v.distance,
	from: $v.from,
	to: $v.to,
	trips: $v.trips
} });
