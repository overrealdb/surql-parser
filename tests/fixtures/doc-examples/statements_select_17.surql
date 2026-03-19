CREATE user SET
    name = "Jack",
    emails = ["my@firstemail.com", "another@builder.com"],
    age = 37;

CREATE user SET
    name = "Ellen",
    emails = ["ruler@forest.com", "wife@tom.com"],
    age = 50;

CREATE user SET
    name = "Phillip",
    emails = ["prior@kingsbridge.com", "boss@remigius.com"],
    age = 50;

SELECT age, emails FROM (SELECT * FROM user SPLIT emails) GROUP BY age;

SELECT age, emails
FROM (
  SELECT age, array::group(emails) AS emails
  FROM user
  GROUP BY age
)
SPLIT emails;
