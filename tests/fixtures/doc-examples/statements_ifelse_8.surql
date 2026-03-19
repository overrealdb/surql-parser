IF $access = 'admin'
	{
        CREATE admin_user_event SET 
            time = time::now(),
            info = "Admin user activity registered";
		SELECT * FROM admin_data WHERE access_level = 'full';
	}
ELSE IF $access = 'user'
	{
		IF $auth.role = 'premium'
			{
                CREATE premium_user_event SET 
                    time = time::now(),
                    info = "Premium user activity registered";

				IF $auth.subscription_status = 'active'
					{ SELECT * FROM premium_user_data WHERE active = 1 }
				ELSE IF $auth.subscription_status = 'trial'
					{ SELECT * FROM trial_user_data }
				ELSE
					{ SELECT * FROM basic_user_data }
			}
		ELSE IF $auth.role = 'standard'
			{ SELECT * FROM standard_user_data WHERE region = 'US' }
		ELSE IF $auth.role = 'standard' AND $auth.subscription_status = 'active'
			{ SELECT * FROM standard_user_data WHERE region = 'EU' }
		ELSE
			{ SELECT * FROM unauthorized_user_data }
	}
ELSE
	{ SELECT * FROM unknown_access_data };
