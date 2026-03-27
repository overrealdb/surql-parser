-- Set CREATE in event to only trigger when record has `true` for `log_event`
DEFINE EVENT something ON person WHEN $input.log_event = true THEN {
    CREATE log SET at = time::now(), of = $input;
};

-- Set to `false`, does not trigger CREATE
CREATE person:debug SET name = "Billy", log_event = false;
-- Triggers CREATE
CREATE person:real SET name = "Bobby", log_event = true;

SELECT * FROM log;
