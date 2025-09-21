-- Add migration script here

-- Audit Events
CREATE INDEX idx_audit_events_user_id ON audit_events (user_id);

-- Login Attempts
CREATE INDEX idx_login_attempts_user_id ON login_attempts (user_id);

-- Devices
CREATE INDEX idx_devices_user_id ON devices (user_id);

-- Users
CREATE INDEX index_users_username ON users (username);
CREATE INDEX index_users_email ON users (email);
CREATE INDEX index_users_phone_number ON users (phone_number);