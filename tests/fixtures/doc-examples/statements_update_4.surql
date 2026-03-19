-- Remove the company field by setting it to NONE or using the UNSET keyword
UPDATE person:tobie SET company = NONE;

UPDATE person:tobie UNSET company;
