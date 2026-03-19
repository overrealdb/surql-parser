SELECT ->(SELECT *, time::now() AS queried_at FROM knows LIMIT 1) FROM person:one;
