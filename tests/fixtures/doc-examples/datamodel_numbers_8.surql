RELATE dr:evil->bribes->other:character SET dollars = 1_000_000.99;
-- [{ dollars: 1000000.99, id: bribes:4bfld2ukwnja24dzrpw9, in: dr:evil, out: other:character }]

-- Input Korean currency counted in units of 10000, not 1000
RELATE korean:purchaser->buys_house_from->korean:seller
              -- 10억 4천만 5천
    SET amount = 10_4000_5000;
-- [{ amount: 1040005000, id: buys_house_from:9070t2ctgwwg202cpw1z, in: korean:purchaser, out: korean:seller }]
