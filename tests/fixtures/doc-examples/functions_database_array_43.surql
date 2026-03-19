INSERT INTO person (
  -- Create 1000 objects with a random ULID and incrementing number
    (<array>0..1000).fold([], |$v, $_, $i| {
    $v.append( { 
      id: rand::ulid(),
      person_num: $i
      });
    })
) RETURN NONE;
