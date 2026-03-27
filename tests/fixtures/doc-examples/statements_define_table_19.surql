DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD user_num ON user TYPE option<int>;

CREATE ONLY user CONTENT { 
    name: "Billy", 
    User_num: 100 // Note: field is capitalized
};
