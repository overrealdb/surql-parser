SELECT 
    id AS country,
    <-in<-province AS provinces,
    <-in<-province<-in<-city AS cities FROM ONLY country:canada;
