use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DomainError, TenantId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub endpoint_url: String,
    pub event_types: Vec<String>,
    pub secret_hash: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl WebhookSubscription {
    pub fn create(
        tenant_id: TenantId,
        endpoint_url: String,
        event_types: Vec<String>,
        secret_hash: String,
    ) -> Result<Self, DomainError> {
        if !endpoint_url.starts_with("https://") {
            return Err(DomainError::validation("webhook endpoint must use HTTPS"));
        }
        if event_types.is_empty() || event_types.iter().any(|event| event.trim().is_empty()) {
            return Err(DomainError::validation(
                "at least one webhook event is required",
            ));
        }
        if secret_hash.len() != 64 || !secret_hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(DomainError::validation("webhook secret hash is invalid"));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            tenant_id,
            endpoint_url,
            event_types,
            secret_hash,
            enabled: true,
            created_at: Utc::now(),
        })
    }
}
