-- Smart Home initial schema
-- Run manually or let the server auto-apply on first connection.

CREATE TABLE IF NOT EXISTS devices (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    device_type TEXT NOT NULL,
    state       TEXT NOT NULL DEFAULT 'off',
    room        TEXT,
    connected   BOOLEAN NOT NULL DEFAULT false,
    last_error  TEXT,
    brightness  SMALLINT NOT NULL DEFAULT 0,
    temperature DOUBLE PRECISION,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
