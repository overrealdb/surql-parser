SELECT
    id AS country,
    <-in<-province AS province,
    <-in<-province<-in<-city AS city FROM country:canada
    SPLIT city, province;
