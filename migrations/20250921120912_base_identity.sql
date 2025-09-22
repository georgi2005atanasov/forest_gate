-- -- Add migration script here
-- CREATE TABLE recovery_codes (
--     user_id BIGINT PRIMARY KEY,
--     code TEXT NOT NULL,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     CONSTRAINT fk_recovery_codes_user
--         FOREIGN KEY (user_id) REFERENCES users(id)
--         ON DELETE CASCADE
-- );

-- CREATE TABLE roles (
--     id SERIAL PRIMARY KEY,
--     name TEXT NOT NULL
-- );

-- CREATE TABLE users_roles (
--     user_id BIGINT NOT NULL,
--     role_id INT NOT NULL,
--     assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     PRIMARY KEY (user_id, role_id),
--     CONSTRAINT fk_users_roles_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
--     CONSTRAINT fk_users_roles_role FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
-- );

-- CREATE TYPE device_type_enum AS ENUM ('unknown', 'mobile', 'tablet', 'pc');
-- CREATE TYPE device_status_enum AS ENUM ('active', 'inactive', 'blocked');

-- CREATE TABLE devices (
--     id BIGSERIAL PRIMARY KEY,
--     user_id BIGINT,
--     os_name TEXT,
--     os_version TEXT,
--     locale TEXT, -- should be enum
--     device_type device_type_enum NOT NULL DEFAULT 'unknown',
--     device_status device_status_enum NOT NULL DEFAULT 'active',
--     app_version TEXT,
--     fingerprint TEXT,
--     extra_data JSONB,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     deleted_at TIMESTAMPTZ,
--     FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
-- );

-- CREATE TYPE session_status_enum AS ENUM ('active', 'expired', 'terminated');

-- CREATE TABLE sessions (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id BIGINT NOT NULL,
--     device_id BIGINT,
--     status session_status_enum NOT NULL DEFAULT 'active',
--     ip_address INET,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     expires_at TIMESTAMPTZ NOT NULL,
--     CONSTRAINT fk_sessions_device
--         FOREIGN KEY (device_id) REFERENCES devices(id) 
--         ON DELETE SET NULL,
--     CONSTRAINT fk_sessions_user
--         FOREIGN KEY (user_id) REFERENCES users(id) 
--         ON DELETE CASCADE
-- );

-- CREATE TYPE key_operation_enum AS ENUM ('auth', 'decryption', 'encryption');

-- CREATE TABLE device_keys (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     device_id BIGINT NOT NULL,
--     public_key BYTEA NOT NULL,
--     operation_usage key_operation_enum NOT NULL,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     revoked_at TIMESTAMPTZ,
--     deleted_at TIMESTAMPTZ,
--     CONSTRAINT fk_device_keys_device
--         FOREIGN KEY (device_id) REFERENCES devices(id) 
--         ON DELETE CASCADE
-- );

-- CREATE TABLE user_interactions (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id BIGINT NOT NULL,
--     session_id UUID NOT NULL,
--     device_id BIGINT,
--     actions JSONB,
--     summary TEXT,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     CONSTRAINT fk_user_interactions_device
--         FOREIGN KEY (device_id) REFERENCES devices(id) 
--         ON DELETE CASCADE,
--     CONSTRAINT fk_user_interactions_user
--         FOREIGN KEY (user_id) REFERENCES users(id) 
--         ON DELETE CASCADE,
--     CONSTRAINT fk_user_interactions_session
--         FOREIGN KEY (session_id) REFERENCES sessions(id) 
--         ON DELETE SET NULL
-- );

-- ALTER TABLE audit_events
--     ADD CONSTRAINT fk_audit_events_session
--     FOREIGN KEY (session_id)
--     REFERENCES sessions(id)
--     ON DELETE SET NULL;

-- ALTER TABLE audit_events
--     ADD CONSTRAINT fk_audit_events_user
--     FOREIGN KEY (user_id)
--     REFERENCES users(id)
--     ON DELETE CASCADE;

-- CREATE TABLE login_attempts (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id BIGINT,
--     success BOOLEAN NOT NULL,
--     ip_address INET,
--     country TEXT,
--     city TEXT,
--     asn TEXT,
--     latitude NUMERIC(9, 6),
--     longitude NUMERIC(9, 6),
--     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
--     CONSTRAINT fk_login_attempts_user
--         FOREIGN KEY (user_id) REFERENCES users(id) 
--         ON DELETE SET NULL
-- );

-- ALTER TABLE roles ADD CONSTRAINT uq_roles_name UNIQUE (name);