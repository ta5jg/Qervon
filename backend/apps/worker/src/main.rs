/* =============================================================================
 * File:           backend/apps/worker/src/main.rs
 * Project:        Qervon
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-05
 * Version:        0.1.0
 *
 * Description:
 *   Defines the executable entry point for the Qervon Worker application.
 *
 * Specification:
 *   QAS-000001 through QAS-000006, QES-000002, QES-000006.
 *
 * License:
 *   Qervon License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

use std::{collections::BTreeSet, env, str::FromStr, time::Duration as StdDuration};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use reqwest::{Client, Url};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use thiserror::Error;
use tokio::time::sleep;
use tracing::{info, warn};
use uuid::Uuid;
use web_push::{
    request_builder, ContentEncoding, SubscriptionInfo, VapidSignatureBuilder,
    WebPushMessageBuilder, URL_SAFE_NO_PAD,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
struct WorkerConfig {
    database_url: String,
    encryption_key: Vec<u8>,
    batch_size: i64,
    max_attempts: i32,
    poll_interval: StdDuration,
    request_timeout: StdDuration,
    webhook_allowed_hosts: BTreeSet<String>,
    vapid_private_key: Option<String>,
    vapid_subject: Option<String>,
    run_once: bool,
}

impl WorkerConfig {
    fn from_env() -> Result<Self, WorkerError> {
        let encoded_key = env::var("QERVON_WEBHOOK_ENCRYPTION_KEY")
            .map_err(|_| WorkerError::Config("QERVON_WEBHOOK_ENCRYPTION_KEY is required".into()))?;
        let encryption_key = BASE64.decode(encoded_key).map_err(|_| {
            WorkerError::Config("QERVON_WEBHOOK_ENCRYPTION_KEY must be base64 encoded".into())
        })?;
        if encryption_key.len() != 32 {
            return Err(WorkerError::Config(
                "QERVON_WEBHOOK_ENCRYPTION_KEY must decode to 32 bytes".into(),
            ));
        }

        let webhook_allowed_hosts = env::var("QERVON_WEBHOOK_ALLOWED_HOSTS")
            .unwrap_or_default()
            .split(',')
            .map(|host| host.trim().trim_end_matches('.').to_ascii_lowercase())
            .filter(|host| !host.is_empty())
            .collect();
        let vapid_private_key = env::var("QERVON_WEB_PUSH_VAPID_PRIVATE_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty());
        let vapid_subject = env::var("QERVON_WEB_PUSH_VAPID_SUBJECT")
            .ok()
            .filter(|subject| subject.starts_with("mailto:") || subject.starts_with("https://"));
        if vapid_private_key.is_some() && vapid_subject.is_none() {
            return Err(WorkerError::Config(
                "QERVON_WEB_PUSH_VAPID_SUBJECT must be mailto: or HTTPS when browser push is configured".into(),
            ));
        }
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| WorkerError::Config("DATABASE_URL is required".into()))?,
            encryption_key,
            batch_size: parse_env("QERVON_WORKER_BATCH_SIZE", 50)?,
            max_attempts: parse_env("QERVON_WEBHOOK_MAX_ATTEMPTS", 8)?,
            poll_interval: StdDuration::from_secs(parse_env("QERVON_WORKER_POLL_SECONDS", 5)?),
            request_timeout: StdDuration::from_secs(parse_env(
                "QERVON_OUTBOUND_TIMEOUT_SECONDS",
                10,
            )?),
            webhook_allowed_hosts,
            vapid_private_key,
            vapid_subject,
            run_once: parse_env("QERVON_WORKER_RUN_ONCE", false)?,
        })
    }
}

#[derive(Debug, Error)]
enum WorkerError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("outbound delivery error: {0}")]
    Delivery(String),
}

#[derive(Debug, FromRow)]
struct OutboxEvent {
    id: Uuid,
    tenant_id: Uuid,
    event_type: String,
    aggregate_id: Uuid,
    payload: Value,
    attempts: i32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct WebhookTarget {
    id: Uuid,
    endpoint_url: String,
    encrypted_secret: Vec<u8>,
}

#[derive(Debug)]
struct SignedWebhookDelivery {
    id: Uuid,
    event_outbox_id: Uuid,
    webhook_id: Uuid,
    tenant_id: Uuid,
    endpoint_url: String,
    event_type: String,
    aggregate_id: Uuid,
    payload: Value,
    body: Vec<u8>,
    signature: String,
}

#[derive(Debug, FromRow)]
struct WebhookDelivery {
    id: Uuid,
    endpoint_url: String,
    event_type: String,
    body: Vec<u8>,
    signature: String,
    attempts: i32,
}

#[derive(Debug, FromRow)]
struct PushDelivery {
    id: Uuid,
    endpoint: String,
    p256dh: String,
    auth: String,
    title: String,
    body: String,
    attempts: i32,
}

#[derive(Debug, FromRow)]
struct QueuedPushDelivery {
    notification_id: Uuid,
    subscription_id: Uuid,
    endpoint: String,
    title: String,
    body: String,
}

#[tokio::main]
async fn main() -> Result<(), WorkerError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "qervon_worker=info,tower_http=info".into()),
        )
        .init();

    let config = WorkerConfig::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    let client = Client::builder()
        .timeout(config.request_timeout)
        .user_agent("qervon-webhook-worker/1.0")
        .build()
        .map_err(|error| WorkerError::Config(format!("could not create HTTP client: {error}")))?;

    loop {
        let processed = run_once(&pool, &config, &client).await?;
        if processed > 0 {
            info!(processed, "processed webhook outbox events");
        }
        if config.run_once {
            return Ok(());
        }
        sleep(config.poll_interval).await;
    }
}

async fn run_once(
    pool: &PgPool,
    config: &WorkerConfig,
    client: &Client,
) -> Result<usize, WorkerError> {
    let claim_token = Uuid::now_v7();
    let events = claim_outbox_events(pool, claim_token, config.batch_size).await?;
    let mut processed = 0;

    for event in events {
        match fanout_event(pool, config, &event).await {
            Ok(delivery_count) => {
                mark_event_fanned_out(pool, event.id).await?;
                info!(
                    event_id = %event.id,
                    delivery_count,
                    "webhook event fanned out"
                );
            }
            Err(error) => {
                warn!(event_id = %event.id, error = %error, "webhook event fanout failed");
                retry_or_dead_letter_event(pool, &event, config.max_attempts, &error.to_string())
                    .await?;
            }
        }
        processed += 1;
    }

    processed += deliver_pending_webhooks(pool, config, client).await?;
    processed += deliver_pending_push_notifications(pool, config, client).await?;
    Ok(processed)
}

async fn fanout_event(
    pool: &PgPool,
    config: &WorkerConfig,
    event: &OutboxEvent,
) -> Result<usize, WorkerError> {
    let targets = load_webhook_targets(pool, event).await?;
    if targets.is_empty() {
        return Ok(0);
    }

    let mut delivery_count = 0;
    for target in targets {
        let secret = decrypt_webhook_secret(&config.encryption_key, &target.encrypted_secret)?;
        let delivery = build_signed_delivery(event, &target, &secret)?;
        insert_webhook_delivery(pool, &delivery).await?;
        delivery_count += 1;
    }

    Ok(delivery_count)
}

async fn claim_outbox_events(
    pool: &PgPool,
    claim_token: Uuid,
    batch_size: i64,
) -> Result<Vec<OutboxEvent>, WorkerError> {
    let events = sqlx::query_as::<_, OutboxEvent>(
        "UPDATE integrations.event_outbox
         SET claimed_at = now(), claim_token = $1
         WHERE id IN (
             SELECT id
             FROM integrations.event_outbox
             WHERE delivered_at IS NULL
               AND dead_lettered_at IS NULL
               AND available_at <= now()
               AND (claimed_at IS NULL OR claimed_at < now() - interval '5 minutes')
             ORDER BY available_at ASC, created_at ASC
             FOR UPDATE SKIP LOCKED
             LIMIT $2
         )
         RETURNING id, tenant_id, event_type, aggregate_id, payload, attempts, created_at",
    )
    .bind(claim_token)
    .bind(batch_size)
    .fetch_all(pool)
    .await?;

    Ok(events)
}

async fn load_webhook_targets(
    pool: &PgPool,
    event: &OutboxEvent,
) -> Result<Vec<WebhookTarget>, WorkerError> {
    let targets = sqlx::query_as::<_, WebhookTarget>(
        "SELECT id, endpoint_url, encrypted_secret
         FROM integrations.webhooks
         WHERE tenant_id = $1
           AND enabled = true
           AND encrypted_secret IS NOT NULL
           AND $2 = ANY(event_types)
         ORDER BY created_at ASC",
    )
    .bind(event.tenant_id)
    .bind(&event.event_type)
    .fetch_all(pool)
    .await?;

    Ok(targets)
}

fn build_signed_delivery(
    event: &OutboxEvent,
    target: &WebhookTarget,
    secret: &[u8],
) -> Result<SignedWebhookDelivery, WorkerError> {
    let body = json!({
        "id": event.id,
        "type": event.event_type,
        "tenant_id": event.tenant_id,
        "aggregate_id": event.aggregate_id,
        "created_at": event.created_at,
        "payload": event.payload,
    });
    let body_bytes = serde_json::to_vec(&body)?;

    Ok(SignedWebhookDelivery {
        id: Uuid::now_v7(),
        event_outbox_id: event.id,
        webhook_id: target.id,
        tenant_id: event.tenant_id,
        endpoint_url: target.endpoint_url.clone(),
        event_type: event.event_type.clone(),
        aggregate_id: event.aggregate_id,
        payload: event.payload.clone(),
        body: body_bytes.clone(),
        signature: sign_body(secret, &body_bytes),
    })
}

async fn insert_webhook_delivery(
    pool: &PgPool,
    delivery: &SignedWebhookDelivery,
) -> Result<(), WorkerError> {
    sqlx::query(
        "INSERT INTO integrations.webhook_delivery_outbox
            (id, event_outbox_id, webhook_id, tenant_id, endpoint_url, event_type,
             aggregate_id, payload, body, signature)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
         ON CONFLICT (event_outbox_id, webhook_id) DO UPDATE
         SET endpoint_url = EXCLUDED.endpoint_url,
             payload = EXCLUDED.payload,
             body = EXCLUDED.body,
             signature = EXCLUDED.signature,
             last_error = NULL",
    )
    .bind(delivery.id)
    .bind(delivery.event_outbox_id)
    .bind(delivery.webhook_id)
    .bind(delivery.tenant_id)
    .bind(&delivery.endpoint_url)
    .bind(&delivery.event_type)
    .bind(delivery.aggregate_id)
    .bind(&delivery.payload)
    .bind(&delivery.body)
    .bind(&delivery.signature)
    .execute(pool)
    .await?;

    Ok(())
}

async fn mark_event_fanned_out(pool: &PgPool, event_id: Uuid) -> Result<(), WorkerError> {
    sqlx::query(
        "UPDATE integrations.event_outbox
         SET delivered_at = now(),
             claimed_at = NULL,
             claim_token = NULL,
             last_error = NULL
         WHERE id = $1",
    )
    .bind(event_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn deliver_pending_webhooks(
    pool: &PgPool,
    config: &WorkerConfig,
    client: &Client,
) -> Result<usize, WorkerError> {
    let deliveries = claim_webhook_deliveries(pool, Uuid::now_v7(), config.batch_size).await?;
    for delivery in &deliveries {
        let result = send_webhook_delivery(client, config, delivery).await;
        match result {
            Ok(()) => mark_webhook_delivery_sent(pool, delivery.id).await?,
            Err(error) => {
                warn!(delivery_id = %delivery.id, error = %error, "webhook delivery failed");
                retry_or_dead_letter_webhook(
                    pool,
                    delivery,
                    config.max_attempts,
                    &error.to_string(),
                )
                .await?;
            }
        }
    }
    Ok(deliveries.len())
}

async fn claim_webhook_deliveries(
    pool: &PgPool,
    claim_token: Uuid,
    batch_size: i64,
) -> Result<Vec<WebhookDelivery>, WorkerError> {
    Ok(sqlx::query_as::<_, WebhookDelivery>(
        "UPDATE integrations.webhook_delivery_outbox
         SET claimed_at = now(), claim_token = $1
         WHERE id IN (
           SELECT id FROM integrations.webhook_delivery_outbox
           WHERE delivered_at IS NULL AND dead_lettered_at IS NULL AND available_at <= now()
             AND (claimed_at IS NULL OR claimed_at < now() - interval '5 minutes')
           ORDER BY available_at ASC, created_at ASC FOR UPDATE SKIP LOCKED LIMIT $2
         )
         RETURNING id, endpoint_url, event_type, body, signature, attempts",
    )
    .bind(claim_token)
    .bind(batch_size)
    .fetch_all(pool)
    .await?)
}

async fn send_webhook_delivery(
    client: &Client,
    config: &WorkerConfig,
    delivery: &WebhookDelivery,
) -> Result<(), WorkerError> {
    let endpoint = checked_webhook_endpoint(&delivery.endpoint_url, &config.webhook_allowed_hosts)?;
    let response = client
        .post(endpoint)
        .header("content-type", "application/json")
        .header("x-qervon-event", &delivery.event_type)
        .header("x-qervon-delivery", delivery.id.to_string())
        .header("x-qervon-signature", &delivery.signature)
        .body(delivery.body.clone())
        .send()
        .await
        .map_err(|error| WorkerError::Delivery(format!("HTTP request failed: {error}")))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(WorkerError::Delivery(format!(
            "endpoint returned HTTP {}",
            response.status()
        )))
    }
}

fn checked_webhook_endpoint(
    value: &str,
    allowed_hosts: &BTreeSet<String>,
) -> Result<Url, WorkerError> {
    let endpoint =
        Url::parse(value).map_err(|_| WorkerError::Delivery("endpoint URL is invalid".into()))?;
    let host = endpoint
        .host_str()
        .map(|host| host.trim_end_matches('.').to_ascii_lowercase())
        .ok_or_else(|| WorkerError::Delivery("endpoint host is missing".into()))?;
    if endpoint.scheme() != "https" || endpoint.username() != "" || endpoint.password().is_some() {
        return Err(WorkerError::Delivery(
            "endpoint must be a credential-free HTTPS URL".into(),
        ));
    }
    if allowed_hosts.is_empty()
        || !allowed_hosts
            .iter()
            .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
    {
        return Err(WorkerError::Delivery(
            "endpoint host is not in QERVON_WEBHOOK_ALLOWED_HOSTS".into(),
        ));
    }
    Ok(endpoint)
}

async fn mark_webhook_delivery_sent(pool: &PgPool, id: Uuid) -> Result<(), WorkerError> {
    sqlx::query(
        "UPDATE integrations.webhook_delivery_outbox
         SET delivered_at = now(), claimed_at = NULL, claim_token = NULL, last_error = NULL
         WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn retry_or_dead_letter_webhook(
    pool: &PgPool,
    delivery: &WebhookDelivery,
    max_attempts: i32,
    last_error: &str,
) -> Result<(), WorkerError> {
    let attempts = delivery.attempts + 1;
    if attempts >= max_attempts {
        sqlx::query(
            "UPDATE integrations.webhook_delivery_outbox
             SET attempts = $2, dead_lettered_at = now(), claimed_at = NULL, claim_token = NULL, last_error = $3
             WHERE id = $1",
        )
        .bind(delivery.id)
        .bind(attempts)
        .bind(last_error)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE integrations.webhook_delivery_outbox
             SET attempts = $2, available_at = $3, claimed_at = NULL, claim_token = NULL, last_error = $4
             WHERE id = $1",
        )
        .bind(delivery.id)
        .bind(attempts)
        .bind(Utc::now() + Duration::seconds(backoff_seconds(attempts)))
        .bind(last_error)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn deliver_pending_push_notifications(
    pool: &PgPool,
    config: &WorkerConfig,
    client: &Client,
) -> Result<usize, WorkerError> {
    enqueue_push_deliveries(pool).await?;
    let deliveries = claim_push_deliveries(pool, Uuid::now_v7(), config.batch_size).await?;
    for delivery in &deliveries {
        let result = send_push_delivery(config, client, delivery).await;
        match result {
            Ok(()) => mark_push_delivery_sent(pool, delivery.id).await?,
            Err(error) => {
                warn!(delivery_id = %delivery.id, error = %error, "browser push delivery failed");
                retry_or_dead_letter_push(pool, delivery, config.max_attempts, &error.to_string())
                    .await?;
            }
        }
    }
    refresh_push_notification_statuses(pool).await?;
    Ok(deliveries.len())
}

async fn enqueue_push_deliveries(pool: &PgPool) -> Result<(), WorkerError> {
    let queued = sqlx::query_as::<_, QueuedPushDelivery>(
        "SELECT n.id AS notification_id, s.id AS subscription_id, s.endpoint, n.title, n.body
         FROM notifications.notifications n
         JOIN notifications.web_push_subscriptions s ON s.user_id = n.recipient_id
         WHERE n.channel = 'push' AND n.status = 'queued'
           AND NOT EXISTS (SELECT 1 FROM notifications.web_push_delivery_outbox d
                           WHERE d.notification_id = n.id AND d.subscription_id = s.id)",
    )
    .fetch_all(pool)
    .await?;
    for delivery in queued {
        sqlx::query(
            "INSERT INTO notifications.web_push_delivery_outbox
             (id, notification_id, subscription_id, endpoint, title, body)
             VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (notification_id, subscription_id) DO NOTHING",
        )
        .bind(Uuid::now_v7())
        .bind(delivery.notification_id)
        .bind(delivery.subscription_id)
        .bind(delivery.endpoint)
        .bind(delivery.title)
        .bind(delivery.body)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn claim_push_deliveries(
    pool: &PgPool,
    claim_token: Uuid,
    batch_size: i64,
) -> Result<Vec<PushDelivery>, WorkerError> {
    Ok(sqlx::query_as::<_, PushDelivery>(
        "UPDATE notifications.web_push_delivery_outbox d
         SET claimed_at = now(), claim_token = $1
         WHERE d.id IN (
           SELECT id FROM notifications.web_push_delivery_outbox
           WHERE delivered_at IS NULL AND dead_lettered_at IS NULL AND available_at <= now()
             AND (claimed_at IS NULL OR claimed_at < now() - interval '5 minutes')
           ORDER BY available_at ASC, created_at ASC FOR UPDATE SKIP LOCKED LIMIT $2
         )
         RETURNING d.id, d.endpoint,
           (SELECT p256dh FROM notifications.web_push_subscriptions s WHERE s.id = d.subscription_id) AS p256dh,
           (SELECT auth FROM notifications.web_push_subscriptions s WHERE s.id = d.subscription_id) AS auth,
           d.title, d.body, d.attempts",
    )
    .bind(claim_token)
    .bind(batch_size)
    .fetch_all(pool)
    .await?)
}

async fn send_push_delivery(
    config: &WorkerConfig,
    client: &Client,
    delivery: &PushDelivery,
) -> Result<(), WorkerError> {
    let private_key = config.vapid_private_key.as_deref().ok_or_else(|| {
        WorkerError::Delivery("QERVON_WEB_PUSH_VAPID_PRIVATE_KEY is not configured".into())
    })?;
    let subject = config
        .vapid_subject
        .as_deref()
        .expect("validated with private key");
    let subscription = SubscriptionInfo::new(
        delivery.endpoint.as_str(),
        delivery.p256dh.as_str(),
        delivery.auth.as_str(),
    );
    let mut signature =
        VapidSignatureBuilder::from_base64(private_key, URL_SAFE_NO_PAD, &subscription).map_err(
            |error| WorkerError::Delivery(format!("invalid VAPID private key: {error}")),
        )?;
    signature.add_claim("sub", subject);
    let mut builder = WebPushMessageBuilder::new(&subscription);
    let payload = serde_json::to_vec(&json!({
        "title": delivery.title,
        "body": delivery.body,
        "url": "/mobile-customer"
    }))?;
    builder.set_payload(ContentEncoding::Aes128Gcm, &payload);
    builder.set_vapid_signature(
        signature.build().map_err(|error| {
            WorkerError::Delivery(format!("could not sign browser push: {error}"))
        })?,
    );
    let request = request_builder::build_request::<Vec<u8>>(builder.build().map_err(|error| {
        WorkerError::Delivery(format!("could not encrypt browser push: {error}"))
    })?);
    let (parts, body) = request.into_parts();
    let mut request = client.post(parts.uri.to_string()).body(body);
    for (name, value) in &parts.headers {
        let value = value
            .to_str()
            .map_err(|_| WorkerError::Delivery("push header is not valid ASCII".into()))?;
        request = request.header(name.as_str(), value);
    }
    let response = request
        .send()
        .await
        .map_err(|error| WorkerError::Delivery(format!("push service request failed: {error}")))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(WorkerError::Delivery(format!(
            "push service returned HTTP {}",
            response.status()
        )))
    }
}

async fn mark_push_delivery_sent(pool: &PgPool, id: Uuid) -> Result<(), WorkerError> {
    sqlx::query(
        "UPDATE notifications.web_push_delivery_outbox
         SET delivered_at = now(), claimed_at = NULL, claim_token = NULL, last_error = NULL WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn retry_or_dead_letter_push(
    pool: &PgPool,
    delivery: &PushDelivery,
    max_attempts: i32,
    last_error: &str,
) -> Result<(), WorkerError> {
    let attempts = delivery.attempts + 1;
    if attempts >= max_attempts {
        sqlx::query(
            "UPDATE notifications.web_push_delivery_outbox
             SET attempts = $2, dead_lettered_at = now(), claimed_at = NULL, claim_token = NULL, last_error = $3 WHERE id = $1",
        )
        .bind(delivery.id).bind(attempts).bind(last_error).execute(pool).await?;
    } else {
        sqlx::query(
            "UPDATE notifications.web_push_delivery_outbox
             SET attempts = $2, available_at = $3, claimed_at = NULL, claim_token = NULL, last_error = $4 WHERE id = $1",
        )
        .bind(delivery.id)
        .bind(attempts)
        .bind(Utc::now() + Duration::seconds(backoff_seconds(attempts)))
        .bind(last_error)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn refresh_push_notification_statuses(pool: &PgPool) -> Result<(), WorkerError> {
    sqlx::query(
        "UPDATE notifications.notifications n SET status = 'sent', sent_at = now()
         WHERE n.channel = 'push' AND n.status = 'queued'
           AND EXISTS (SELECT 1 FROM notifications.web_push_delivery_outbox d WHERE d.notification_id = n.id)
           AND NOT EXISTS (SELECT 1 FROM notifications.web_push_delivery_outbox d
                           WHERE d.notification_id = n.id AND d.delivered_at IS NULL AND d.dead_lettered_at IS NULL)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn retry_or_dead_letter_event(
    pool: &PgPool,
    event: &OutboxEvent,
    max_attempts: i32,
    last_error: &str,
) -> Result<(), WorkerError> {
    let attempts = event.attempts + 1;
    if attempts >= max_attempts {
        sqlx::query(
            "UPDATE integrations.event_outbox
             SET attempts = $2,
                 dead_lettered_at = now(),
                 claimed_at = NULL,
                 claim_token = NULL,
                 last_error = $3
             WHERE id = $1",
        )
        .bind(event.id)
        .bind(attempts)
        .bind(last_error)
        .execute(pool)
        .await?;
    } else {
        let available_at = Utc::now() + Duration::seconds(backoff_seconds(attempts));
        sqlx::query(
            "UPDATE integrations.event_outbox
             SET attempts = $2,
                 available_at = $3,
                 claimed_at = NULL,
                 claim_token = NULL,
                 last_error = $4
             WHERE id = $1",
        )
        .bind(event.id)
        .bind(attempts)
        .bind(available_at)
        .bind(last_error)
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn decrypt_webhook_secret(
    encryption_key: &[u8],
    encrypted_secret: &[u8],
) -> Result<Vec<u8>, WorkerError> {
    if encrypted_secret.len() <= 12 {
        return Err(WorkerError::Crypto(
            "encrypted webhook secret is too short".into(),
        ));
    }
    let cipher = Aes256Gcm::new_from_slice(encryption_key)
        .map_err(|_| WorkerError::Crypto("invalid webhook encryption key".into()))?;
    let (nonce_bytes, ciphertext) = encrypted_secret.split_at(12);
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| WorkerError::Crypto("could not decrypt webhook secret".into()))
}

fn sign_body(secret: &[u8], body: &[u8]) -> String {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(secret)
        .expect("HMAC-SHA256 accepts keys of any length");
    mac.update(body);
    format!("sha256={}", bytes_to_hex(&mac.finalize().into_bytes()))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn backoff_seconds(attempts: i32) -> i64 {
    let exponent = attempts.clamp(1, 8) as u32 - 1;
    30_i64
        .saturating_mul(2_i64.saturating_pow(exponent))
        .min(3_600)
}

fn parse_env<T>(name: &str, default: T) -> Result<T, WorkerError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|error| WorkerError::Config(format!("{name} is invalid: {error}"))),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(WorkerError::Config(format!("{name} is invalid: {error}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_body_with_hmac_sha256() {
        let secret = vec![0x0b; 20];
        let signature = sign_body(&secret, b"Hi There");

        assert_eq!(
            signature,
            "sha256=b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn backoff_grows_and_caps() {
        assert_eq!(backoff_seconds(1), 30);
        assert_eq!(backoff_seconds(2), 60);
        assert_eq!(backoff_seconds(4), 240);
        assert_eq!(backoff_seconds(20), 3_600);
    }

    #[test]
    fn decrypts_nonce_prefixed_aes_gcm_secret() {
        let key = [7_u8; 32];
        let secret = b"qvwh_test_secret";
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let nonce = Nonce::from_slice(&[3_u8; 12]);
        let mut encrypted = nonce.to_vec();
        encrypted.extend(cipher.encrypt(nonce, secret.as_ref()).unwrap());

        assert_eq!(decrypt_webhook_secret(&key, &encrypted).unwrap(), secret);
    }

    #[test]
    fn signs_the_exact_persisted_delivery_bytes() {
        let event = OutboxEvent {
            id: Uuid::nil(),
            tenant_id: Uuid::from_u128(1),
            event_type: "order.delivered".into(),
            aggregate_id: Uuid::from_u128(2),
            payload: json!({"status": "delivered"}),
            attempts: 0,
            created_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        };
        let target = WebhookTarget {
            id: Uuid::from_u128(3),
            endpoint_url: "https://hooks.example.test/qervon".into(),
            encrypted_secret: vec![],
        };

        let delivery = build_signed_delivery(&event, &target, b"secret").unwrap();

        assert_eq!(delivery.signature, sign_body(b"secret", &delivery.body));
        assert_eq!(
            serde_json::from_slice::<Value>(&delivery.body).unwrap()["type"],
            "order.delivered"
        );
    }

    #[test]
    fn webhook_allowlist_accepts_only_https_approved_hosts() {
        let allowed = BTreeSet::from(["hooks.example.test".to_string()]);
        assert!(checked_webhook_endpoint("https://hooks.example.test/qervon", &allowed).is_ok());
        assert!(
            checked_webhook_endpoint("https://tenant.hooks.example.test/qervon", &allowed).is_ok()
        );
        assert!(
            checked_webhook_endpoint("https://hooks.example.test.evil.test/qervon", &allowed)
                .is_err()
        );
        assert!(checked_webhook_endpoint("http://hooks.example.test/qervon", &allowed).is_err());
        assert!(
            checked_webhook_endpoint("https://hooks.example.test@evil.test/qervon", &allowed)
                .is_err()
        );
    }
}
