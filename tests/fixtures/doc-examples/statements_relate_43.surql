RELATE city:calgary->sister_of->city:daejeon; -- OK
RELATE city:daejeon->sister_of->city:calgary;
-- "Database index `only_one_sister_city` already contains '[city:calgary, city:daejeon]', with record `sister_of:npab0uoxogmrvpwsvfoa`"
