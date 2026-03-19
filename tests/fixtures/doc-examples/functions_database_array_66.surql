INSERT INTO person ((<array>0..=1000).map(|| {id: rand::ulid()}));
