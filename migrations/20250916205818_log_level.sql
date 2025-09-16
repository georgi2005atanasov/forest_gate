-- Add migration script here
CREATE TYPE log_level_enum AS ENUM (
    'debug',
    'info',
    'warn',
    'error',
    'critical'
);
