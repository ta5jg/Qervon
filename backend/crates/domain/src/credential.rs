use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{TenantId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    pub user_id: UserId,
    pub password_hash: String,
    pub password_changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshSession {
    pub id: Uuid,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl RefreshSession {
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}
