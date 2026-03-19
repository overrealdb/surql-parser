DEFINE ANALYZER error_filter FILTERS mapper('error_filter.txt');
DEFINE INDEX OVERWRITE errors ON TABLE error FIELDS message FULLTEXT ANALYZER error_filter;

FOR $message IN ["File not found", "Datei nicht gefunden", "Zeitüberschreitung"] {
	CREATE error SET message = $message.replace(' ', '_'), at = time::now();
};

SELECT * FROM error WHERE message @@ "NOT_FOUND";
