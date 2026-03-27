DEFINE FUNCTION fn::round::up($num: number) {
    IF $num % 2 == 0 {
        RETURN $num; -- Breaks execution for the function
    };

    -- This is only executed if the RETURN inside the IF statement did not break execution
    RETURN $num + 1;
};
