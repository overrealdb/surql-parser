UPSERT person SET railcard =
    IF age <= 10 { 'junior' }
    ELSE IF age <= 21 { 'student' }
    ELSE IF age >= 65 { 'senior' }
    ELSE { NULL };
