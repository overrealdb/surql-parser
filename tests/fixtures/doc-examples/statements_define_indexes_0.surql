-- optimise queries looking for users of a given age
DEFINE INDEX userAgeIndex ON TABLE user COLUMNS age;
