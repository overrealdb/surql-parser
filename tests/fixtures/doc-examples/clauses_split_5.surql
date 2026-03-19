(SELECT
    id,
    <-in<-province AS province,
    <-in<-province<-in<-city AS city FROM country:canada
    SPLIT city, province)
.map(|$obj| 
    <string>$obj.id.id() 
    + '|' 
    + <string>$obj.province.id() 
    + '|' 
    + <string>$obj.city.id()
);
