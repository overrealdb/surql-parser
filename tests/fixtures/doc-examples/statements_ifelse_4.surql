LET $badly_formatted_datetime = "2024-04TT08:08:08Z";

IF !type::is_datetime($badly_formatted_datetime) {
    THROW "Whoops, that isn't a real datetime"
};
