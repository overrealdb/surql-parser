SELECT ->(SELECT * FROM knows) FROM person:one;
SELECT ->knows.* FROM person:one;
