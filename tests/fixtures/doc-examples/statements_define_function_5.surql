DEFINE FUNCTION fn::age_and_name($user_num: int) -> { age: int, name: string } | string {
    LET $user = type::record("user", $user_num);
    IF $user.exists() {
        $user.{ name, age }
    } ELSE {
        { "Couldn't find user number " + <string>$user_num + "!" }        
    }
};

CREATE user:1 SET name = "Billy", age = 15;

fn::age_and_name(1);
fn::age_and_name(2);
