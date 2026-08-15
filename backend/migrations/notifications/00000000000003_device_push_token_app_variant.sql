-- =============================================================================
-- File:           backend/migrations/notifications/00000000000003_device_push_token_app_variant.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-16
-- Version:        0.1.0
--
-- Description:
--   Adds `app_variant` to device push tokens: the two iOS apps
--   (com.qervon.ios.courier, com.qervon.ios.customer) have distinct bundle
--   identifiers, and APNs rejects a push whose `apns-topic` header does not
--   exactly match the bundle id that issued the device token. Without this
--   column a push provider cannot know which bundle id to address. Defaults
--   to 'courier' for any hypothetical pre-existing row (this table has no
--   real registrations yet in this environment).
--
-- Specification:
--   QAS-000002, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE notifications.device_push_tokens
    ADD COLUMN app_variant text NOT NULL DEFAULT 'courier';

ALTER TABLE notifications.device_push_tokens
    ADD CONSTRAINT device_push_tokens_app_variant_check
    CHECK (app_variant IN ('courier', 'customer'));
