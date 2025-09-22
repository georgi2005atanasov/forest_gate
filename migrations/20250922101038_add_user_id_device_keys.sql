-- -- Add migration script here
-- ALTER TABLE device_keys
--   ADD COLUMN IF NOT EXISTS user_id BIGINT;

-- ALTER TABLE device_keys
--   ADD CONSTRAINT fk_device_keys_user
--   FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- CREATE INDEX IF NOT EXISTS idx_device_keys_user_id ON device_keys(user_id);
