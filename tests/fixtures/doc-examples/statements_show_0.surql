-- Define the change feed and its duration
DEFINE TABLE reading CHANGEFEED 3d;

-- Create some records in the reading table
CREATE reading SET story = "Once upon a time";
CREATE reading SET story = "there was a database";

-- Replay changes to the reading table since a date
SHOW CHANGES FOR TABLE reading SINCE d"2023-09-07T01:23:52Z" LIMIT 10;
-- Replay changes to the reading table since a versionstamp
SHOW CHANGES FOR TABLE reading SINCE 1 LIMIT 10;
