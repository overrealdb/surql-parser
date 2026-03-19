-- Returns [] because no 4th-level relations exist
planet:earth.{4}->has->(?);
-- Returns `city` records located at depth 3
planet:earth.{1..4}->has->(?);
-- Open-ended range: also returns `city` records at depth 3
planet:earth.{..}->has->(?);
