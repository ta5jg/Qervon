-- Durable browser push subscriptions. Endpoints are owned by a signed user and
-- are invalidated by the worker when a push service returns 404 or 410.
CREATE TABLE notifications.web_push_subscriptions (
    id           uuid PRIMARY KEY,
    user_id      uuid NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    endpoint     text NOT NULL UNIQUE CHECK (endpoint LIKE 'https://%'),
    p256dh       text NOT NULL,
    auth         text NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX notifications_web_push_subscriptions_user_idx
    ON notifications.web_push_subscriptions (user_id, updated_at DESC);

CREATE TABLE notifications.web_push_delivery_outbox (
    id              uuid PRIMARY KEY,
    notification_id uuid NOT NULL REFERENCES notifications.notifications(id) ON DELETE CASCADE,
    subscription_id uuid NOT NULL REFERENCES notifications.web_push_subscriptions(id) ON DELETE CASCADE,
    endpoint        text NOT NULL,
    title           text NOT NULL,
    body            text NOT NULL,
    attempts        integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    available_at    timestamptz NOT NULL DEFAULT now(),
    claimed_at      timestamptz,
    claim_token     uuid,
    delivered_at    timestamptz,
    dead_lettered_at timestamptz,
    last_error      text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (notification_id, subscription_id)
);

CREATE INDEX notifications_web_push_delivery_claimable_idx
    ON notifications.web_push_delivery_outbox (available_at, created_at)
    WHERE delivered_at IS NULL AND dead_lettered_at IS NULL;
