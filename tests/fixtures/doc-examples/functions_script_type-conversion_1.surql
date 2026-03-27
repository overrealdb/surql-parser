CREATE user:test SET
	session_timeout = function() {
		return new Duration('1w');
	},
	best_friend = function() {
		return new Record('user', 'joanna');
	},
	identifier = function() {
		return new Uuid('03412258-988f-47cd-82db-549902cdaffe');
	}
;
