SELECT 
    *,
    ->? AS actions,
    <-? AS was,
    <->? AS involved_in
FROM explorer:drake;
