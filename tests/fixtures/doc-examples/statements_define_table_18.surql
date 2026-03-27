DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD user_num ON user TYPE int;

CREATE ONLY user CONTENT { 
    name: "Billy", 
    user_num: 100,
    unneeded_data: {
        some: "other",
        needless: "data"
    }
};
