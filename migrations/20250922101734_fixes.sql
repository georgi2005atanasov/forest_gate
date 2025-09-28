-- Add migration script here
-- =========================================================
-- Clean init schema (PostgreSQL)
-- =========================================================

-- Extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ======================
-- ENUM types
-- ======================
-- CREATE TYPE log_level_enum AS ENUM (
--   'debug', 'info', 'warn', 'error', 'critical'
-- );

-- CREATE TYPE login_method AS ENUM (
--   'with_password',
--   'with_email',
--   'with_phone_number'
-- );

CREATE TYPE event_type_enum AS ENUM (
  'config_change',
  'login'
);
--
CREATE TYPE device_type_enum AS ENUM ('unknown', 'mobile', 'tablet', 'pc', 'desktop');
--
CREATE TYPE device_status_enum AS ENUM ('active', 'inactive', 'blocked');
--
CREATE TYPE session_status_enum AS ENUM ('active', 'expired', 'terminated');

CREATE TYPE key_operation_enum AS ENUM ('auth', 'decryption', 'encryption');

-- ======================
-- Tables
-- ======================

-- users
CREATE TABLE users (
  id                BIGSERIAL PRIMARY KEY,
  username          TEXT NOT NULL UNIQUE,
  email             TEXT NOT NULL UNIQUE,
  phone_number      TEXT,
  password_hash     TEXT NOT NULL,
  salt              BYTEA NOT NULL,
  is_email_verified BOOLEAN NOT NULL DEFAULT FALSE,
  is_phone_verified BOOLEAN NOT NULL DEFAULT FALSE,
  -- kept TEXT in migrations; if you prefer enum, change to: login_method login_method NOT NULL
  login_method      TEXT NOT NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ
);

-- function + trigger to maintain users.updated_at
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- config
CREATE TABLE config (
  id                              SERIAL PRIMARY KEY,
  allow_recovery_codes            BOOLEAN NOT NULL,
  allow_refresh_tokens            BOOLEAN NOT NULL,
  token_validity_seconds          INT NOT NULL,
  refresh_token_validity_seconds  INT NOT NULL,
  ai_model                        TEXT NOT NULL,
  vector_similarity_threshold     INT NOT NULL
);

-- roles
CREATE TABLE roles (
  id   SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);

-- devices (final state has NO user_id)
CREATE TABLE devices (
  id             BIGSERIAL PRIMARY KEY,
  os_name        TEXT,
  os_version     TEXT,
  locale         TEXT,
  device_type    device_type_enum NOT NULL DEFAULT 'unknown',
  device_status  device_status_enum NOT NULL DEFAULT 'active',
  app_version    TEXT,
  fingerprint    TEXT,
  extra_data     JSONB,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at     TIMESTAMPTZ
);

-- sessions (device_id NOT NULL; ON DELETE CASCADE)
CREATE TABLE sessions (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     BIGINT NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
  device_id   BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  status      session_status_enum NOT NULL DEFAULT 'active',
  ip_address  INET,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at  TIMESTAMPTZ NOT NULL
);

-- user_devices (link table)
CREATE TABLE user_devices (
  user_id    BIGINT NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
  device_id  BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  paired_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  is_primary BOOLEAN NOT NULL DEFAULT FALSE,
  revoked_at TIMESTAMPTZ,
  PRIMARY KEY (user_id, device_id)
);

-- device_keys (also has user_id; FK CASCADE)
CREATE TABLE device_keys (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  device_id        BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  user_id          BIGINT REFERENCES users(id) ON DELETE CASCADE,
  public_key       BYTEA NOT NULL,
  operation_usage  key_operation_enum NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked_at       TIMESTAMPTZ,
  deleted_at       TIMESTAMPTZ
);

-- recovery_codes
CREATE TABLE recovery_codes (
  user_id     BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  code        TEXT NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- users_roles
CREATE TABLE users_roles (
  user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role_id     INT    NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, role_id)
);

-- audit_events
CREATE TABLE audit_events (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  event_type  event_type_enum NOT NULL,
  log_level   log_level_enum NOT NULL,
  session_id  UUID REFERENCES sessions(id) ON DELETE SET NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- login_attempts
CREATE TABLE login_attempts (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     BIGINT REFERENCES users(id) ON DELETE SET NULL,
  success     BOOLEAN NOT NULL,
  ip_address  INET,
  country     TEXT,
  city        TEXT,
  asn         TEXT,
  latitude    NUMERIC(9, 6),
  longitude   NUMERIC(9, 6),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- user_interactions (device_id NOT NULL; session_id ON DELETE CASCADE)
CREATE TABLE user_interactions (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  session_id  UUID   NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  device_id   BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  actions     JSONB,
  summary     TEXT,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ======================
-- Seed data
-- ======================
INSERT INTO config (
  id, allow_recovery_codes, allow_refresh_tokens,
  token_validity_seconds, refresh_token_validity_seconds,
  ai_model, vector_similarity_threshold
) VALUES (
  1, true, true, 15000, 100000, 'my-model', 90
);

-- ======================
-- Indexes (final state)
-- ======================

-- Users
CREATE INDEX index_users_username      ON users (username);
CREATE INDEX index_users_email         ON users (email);
CREATE INDEX index_users_phone_number  ON users (phone_number);

-- Audit Events
CREATE INDEX idx_audit_events_user_id  ON audit_events (user_id);

-- Login Attempts
CREATE INDEX idx_login_attempts_user_id ON login_attempts (user_id);

-- Sessions
CREATE INDEX idx_sessions_device_id ON sessions (device_id);

-- User Interactions
CREATE INDEX idx_user_interactions_device_id ON user_interactions (device_id);

-- Device Keys
CREATE INDEX idx_device_keys_device_id ON device_keys (device_id);
CREATE INDEX idx_device_keys_user_id   ON device_keys (user_id);

-- User Devices
CREATE INDEX idx_user_devices_user_id   ON user_devices (user_id);
CREATE INDEX idx_user_devices_device_id ON user_devices (device_id);

-- Optional hardening: unique active fingerprint
CREATE UNIQUE INDEX ux_devices_fingerprint_active
  ON devices (fingerprint)
  WHERE deleted_at IS NULL;
