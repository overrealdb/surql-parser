RETURN function() {
	// List all posts
	const posts = fetch('https://jsonplaceholder.typicode.com/posts');

	// Update post with ID 1
	const updated = fetch('https://jsonplaceholder.typicode.com/posts/1', {
		method: 'PUT',
		body: JSON.stringify({
			id: 1,
			title: 'foo',
			body: 'bar',
			userId: 1,
		}),
		headers: {
			'Content-type': 'application/json; charset=UTF-8',
		},
	});

	return { posts, updated };
}
