person:you.{..1}.{
    id,
    connections: ->knows->person.@
};

person:you.{..1+path}->knows->person;
