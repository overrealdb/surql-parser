CREATE user:test SET created_at = function() {
	return new Date();
};
