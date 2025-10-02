-- Add migration script here
ALTER TABLE user_interactions
  ALTER COLUMN user_id DROP NOT NULL,
  ALTER COLUMN session_id DROP NOT NULL,
  ALTER COLUMN device_id DROP NOT NULL;