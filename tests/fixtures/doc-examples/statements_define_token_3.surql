-- Create a TOKEN if it does not already exist
DEFINE TOKEN IF NOT EXISTS example ON SCOPE example TYPE HS512 VALUE "example";
