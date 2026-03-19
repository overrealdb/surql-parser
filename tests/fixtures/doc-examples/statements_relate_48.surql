// Output without alias
{
	"->friends_with": {
		"->cat": {
			"->friends_with": {
				"->cat": [
					cat:three
				]
			}
		}
	}
}

// Output with alias
{
	friends_of_friends: [
		cat:three
	]
}
