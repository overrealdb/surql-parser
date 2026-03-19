CREATE film SET
	ratings = [
		{ rating: 6, user: user:bt8e39uh1ouhfm8ko8s0 },
		{ rating: 8, user: user:bsilfhu88j04rgs0ga70 },
	],
	featured = function() {
		return this.ratings.filter(
			({ rating }) => rating >= 7
		).map(({ rating, ...data }) => {
			return {
				...data,
				rating: rating * 10
			};
		});
	}
;
