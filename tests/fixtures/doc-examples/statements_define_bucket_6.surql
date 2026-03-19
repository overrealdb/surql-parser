-- Set permissions for the bucket
DEFINE BUCKET admin_bucket BACKEND "memory"
  PERMISSIONS WHERE $auth.admin = true
