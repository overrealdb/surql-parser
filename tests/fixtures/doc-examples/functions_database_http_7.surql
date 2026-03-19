RETURN http::post('https://jsonplaceholder.typicode.com/posts/', {
  id: 1,
  body: "This is some awesome thinking!",
  postId: 100,
  user: {
    id: 63,
    username: "eburras1q"
  }
}, {
  'Authorization': 'Bearer your-token-here',
  'Content-Type': 'application/json',
  'x-custom-header': 'custom-value'
});
