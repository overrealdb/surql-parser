UPDATE utility:gas   SET connected_to -= house:one;
UPDATE utility:water SET connected_to -= house:one;

DELETE house:one;
