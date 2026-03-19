-- Creates an analyzer suitable for auto-completion.
DEFINE ANALYZER autocomplete FILTERS lowercase,edgengram(2,10);
