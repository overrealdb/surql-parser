CREATE person SET scores = function() {
	return [1,2,3].map(v => v * 10);
};
