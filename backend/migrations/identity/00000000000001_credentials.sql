-- =============================================================================
-- File:           backend/migrations/identity/00000000000001_credentials.sql
-- Project:        Qervon
-- Description:    Stores Argon2 password hashes and refresh-token rotation state.
-- =============================================================================

CREATE TABLE identity.credentials (
    user_id uuid PRIMARY KEY REFERENCES identity.users (id) ON DELETE CASCADE,
    password_hash text NOT NULL,
    password_changed_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE identity.refresh_sessions (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES identity.users (id) ON DELETE CASCADE,
    tenant_id uuid NOT NULL,
    token_hash text NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX refresh_sessions_active_user_idx
    ON identity.refresh_sessions (user_id, expires_at)
    WHERE revoked_at IS NULL;
