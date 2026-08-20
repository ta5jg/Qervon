-- =============================================================================
-- File:           backend/migrations/identity/00000000000004_password_reset_tokens.sql
-- Project:        Qervon
-- Description:    Stores hashed, single-use "forgot password" reset tokens.
--                  Only the SHA-256 hash of the raw token is ever persisted —
--                  the raw token is emailed once and never stored, mirroring
--                  identity.refresh_sessions.
-- =============================================================================

CREATE TABLE identity.password_reset_tokens (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES identity.users (id) ON DELETE CASCADE,
    token_hash text NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    used_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX password_reset_tokens_active_user_idx
    ON identity.password_reset_tokens (user_id, expires_at)
    WHERE used_at IS NULL;
