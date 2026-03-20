DEFINE FUNCTION OVERWRITE fn::greet($name: string) {
	RETURN "Hello, " + $name + "!";
};

DEFINE FUNCTION OVERWRITE fn::user_count() {
	RETURN count(SELECT id FROM user);
};
