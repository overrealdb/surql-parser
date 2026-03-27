RETURN http::delete('https://jsonplaceholder.typicode.com/posts/1', {
	'x-my-header': 'some unique string'
});

{}
