-- -- Add migration script here
-- CREATE TABLE IF NOT EXISTS user_devices (
--     user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
--     device_id   BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
--     paired_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
--     is_primary  BOOLEAN NOT NULL DEFAULT FALSE,
--     revoked_at  TIMESTAMPTZ,
--     PRIMARY KEY (user_id, device_id)
-- );

-- CREATE INDEX IF NOT EXISTS idx_user_devices_user_id   ON user_devices (user_id);
-- CREATE INDEX IF NOT EXISTS idx_user_devices_device_id ON user_devices (device_id);

-- DROP INDEX IF EXISTS idx_devices_user_id;
-- ALTER TABLE devices
--     DROP CONSTRAINT IF EXISTS devices_user_id_fkey,
--     DROP COLUMN IF EXISTS user_id;

-- CREATE OR REPLACE FUNCTION set_updated_at()
-- RETURNS trigger AS $$
-- BEGIN
--   NEW.updated_at = now();
--   RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- DROP TRIGGER IF EXISTS trg_users_updated_at ON users;

-- CREATE TRIGGER trg_users_updated_at
-- BEFORE UPDATE ON users
-- FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- DROP INDEX IF EXISTS idx_devices_user_id;

-- ALTER TABLE devices
--   DROP CONSTRAINT IF EXISTS devices_user_id_fkey;

-- ALTER TABLE devices
--   DROP COLUMN IF EXISTS user_id;

-- -- 4) Enforce device_id NOT NULL and correct FK actions

-- -- 4a) sessions: device_id NOT NULL, FK ON DELETE CASCADE (not SET NULL)
-- ALTER TABLE sessions
--   DROP CONSTRAINT IF EXISTS fk_sessions_device;

-- ALTER TABLE sessions
--   ALTER COLUMN device_id SET NOT NULL;

-- ALTER TABLE sessions
--   ADD CONSTRAINT fk_sessions_device
--   FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE;

-- CREATE INDEX IF NOT EXISTS idx_sessions_device_id ON sessions(device_id);

-- -- 4b) user_interactions: device_id NOT NULL, FK ON DELETE CASCADE
-- ALTER TABLE user_interactions
--   DROP CONSTRAINT IF EXISTS fk_user_interactions_device;

-- ALTER TABLE user_interactions
--   ALTER COLUMN device_id SET NOT NULL;

-- ALTER TABLE user_interactions
--   ADD CONSTRAINT fk_user_interactions_device
--   FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE;

-- CREATE INDEX IF NOT EXISTS idx_user_interactions_device_id ON user_interactions(device_id);

-- -- 4c) user_interactions: session_id is NOT NULL in your schema,
-- --     so FK cannot be ON DELETE SET NULL. Use CASCADE.
-- ALTER TABLE user_interactions
--   DROP CONSTRAINT IF EXISTS fk_user_interactions_session;

-- ALTER TABLE user_interactions
--   ADD CONSTRAINT fk_user_interactions_session
--   FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE;

-- -- 4d) device_keys: FK already NOT NULL with ON DELETE CASCADE; just ensure index exists
-- CREATE INDEX IF NOT EXISTS idx_device_keys_device_id ON device_keys(device_id);

-- -- 5) Optional hardening: unique active fingerprint (if desired)
-- CREATE UNIQUE INDEX IF NOT EXISTS ux_devices_fingerprint_active
--   ON devices (fingerprint)
--   WHERE deleted_at IS NULL;