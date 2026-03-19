DEFINE ANALYZER error_filter FILTERS mapper('error_filter.txt');

LET $messages = 
	["File not found", "Datei nicht gefunden", "Zeitüberschreitung"]
	.map(|$word| $word.replace(' ', '_'))
	.join(' ');
search::analyze("error_filter", $messages);
