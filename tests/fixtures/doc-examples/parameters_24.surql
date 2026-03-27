DEFINE FUNCTION fn::get_timeout() -> duration {
    // Do some HTTP call to get status
    // Simulate the output with rand::enum() function
    rand::enum(100ms, 1s, 5s)
};

SELECT * FROM person TIMEOUT fn::get_timeout();
