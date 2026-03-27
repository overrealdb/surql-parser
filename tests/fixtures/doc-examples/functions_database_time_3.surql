// Return the const
RETURN time::epoch;
-- d'1970-01-01T00:00:00Z'

// Define field using the const
DEFINE FIELD since_epoch ON event COMPUTED time::now().floor(1d) - time::epoch;
CREATE ONLY event:one SET information = "Something happened";
-- { id: event:one, information: 'Something happened', since_epoch: 55y42w6d }
