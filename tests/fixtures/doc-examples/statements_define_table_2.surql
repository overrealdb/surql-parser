-- Define the change feed and its duration
-- Optionally, append INCLUDE ORIGINAL to include info
-- on the current record before a change took place
DEFINE TABLE reading CHANGEFEED 3d;

-- Create some records in the reading table
CREATE reading SET story = "Once upon a time";
CREATE reading SET story = "there was a database";

-- Replay changes to the reading table since a certain date
-- Must be after the timestamp at which the changefeed began
SHOW CHANGES FOR TABLE reading SINCE d"2023-09-07T01:23:52Z" LIMIT 10;

-- Alternatively, show the changes for the table since a version number
SHOW CHANGES FOR TABLE reading SINCE 0 LIMIT 10;
