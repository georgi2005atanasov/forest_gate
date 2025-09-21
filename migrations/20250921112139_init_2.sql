CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- CREATE TYPE log_level_enum AS ENUM (
--     'debug',
--     'info',
--     'warn',
--     'error',
--     'critical'
-- );
-- CREATE TYPE login_method AS ENUM (
--     'with_password', 
--     'with_email', 
--     'with_phone_number'
-- );
-- CREATE TYPE event_type_enum AS ENUM (
--     'config_change',
--     'login'
-- );
-- CREATE TYPE log_level_enum AS ENUM (
--     'debug',
--     'info',
--     'warn',
--     'error',
--     'critical'
-- );

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    phone_number TEXT,
    password_hash TEXT NOT NULL,
    salt BYTEA NOT NULL,
    is_email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_phone_verified BOOLEAN NOT NULL DEFAULT FALSE,
    login_method TEXT NOT NULL, -- you can later make this an ENUM
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE config (
    id SERIAL PRIMARY KEY,
    allow_recovery_codes BOOLEAN NOT NULL,
    allow_refresh_tokens BOOLEAN NOT NULL,
    token_validity_seconds INT NOT NULL,
    refresh_token_validity_seconds INT NOT NULL,
    ai_model TEXT NOT NULL,
    vector_similarity_threshold INT NOT NULL
);
-- seeding for config
INSERT INTO config (
    id,
    allow_recovery_codes,
    allow_refresh_tokens,
    token_validity_seconds,
    refresh_token_validity_seconds,
    ai_model,
    vector_similarity_threshold
) VALUES (
    1, true, true, 15000, 100000, 'my-model', 90
);

CREATE TABLE audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id BIGINT NOT NULL,
    event_type event_type_enum NOT NULL,
    log_level log_level_enum NOT NULL,
    session_id UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);