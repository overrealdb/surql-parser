DEFINE TABLE pc;
DEFINE TABLE pc_agg AS SELECT count(), class FROM pc GROUP BY class;
CREATE |pc:3| SET class = "Wizard";
CREATE |pc:10| SET class = "Warrior";
SELECT * FROM pc_agg;
-- Error: pc_agg requires pc to work
REMOVE TABLE pc;
REMOVE TABLE pc_agg;
-- pc_agg is now gone, pc can be removed too
REMOVE TABLE pc;
