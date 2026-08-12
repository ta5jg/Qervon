-- =============================================================================
-- File:           backend/migrations/notifications/00000000000002_device_push_tokens.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Native mobile push device token registration (iOS/Android). This only
--   records where a push COULD be delivered; no APNs/FCM sending is wired
--   yet (see BACKEND_BACKLOG.md).
--
-- Specification:
--   QAS-000002, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE TABLE notifications.device_push_tokens (
    id            uuid PRIMARY KEY,
    user_id       uuid NOT NULL REFERENCES identity.users (id) ON DELETE CASCADE,
    platform      text NOT NULL,
    device_token  text NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT device_push_tokens_platform_check CHECK (platform IN ('ios', 'android')),
    CONSTRAINT device_push_tokens_user_token_unique UNIQUE (user_id, device_token)
);

CREATE INDEX device_push_tokens_user_idx ON notifications.device_push_tokens (user_id);
