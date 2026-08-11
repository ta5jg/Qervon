-- =============================================================================
-- File:           backend/migrations/notifications/00000000000000_initial.sql
-- Project:        Qervon
-- Description:    Persistent notification lifecycle storage.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS notifications;

CREATE TABLE notifications.notifications (
    id           uuid PRIMARY KEY,
    recipient_id uuid NOT NULL,
    channel      text NOT NULL,
    title        text NOT NULL,
    body         text NOT NULL,
    status       text NOT NULL,
    created_at   timestamptz NOT NULL,
    sent_at      timestamptz,
    CONSTRAINT notifications_channel_check
        CHECK (channel IN ('push', 'sms', 'email', 'whatsapp')),
    CONSTRAINT notifications_status_check
        CHECK (status IN ('queued', 'sent', 'failed', 'read')),
    CONSTRAINT notifications_title_check CHECK (length(trim(title)) > 0),
    CONSTRAINT notifications_body_check CHECK (length(trim(body)) > 0)
);

CREATE INDEX notifications_recipient_idx
    ON notifications.notifications (recipient_id, created_at DESC);
