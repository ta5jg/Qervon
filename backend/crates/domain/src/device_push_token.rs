// =============================================================================
// File:           backend/crates/domain/src/device_push_token.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Native mobile push device registration. This only records a device
//   token so the platform knows where a future push notification would be
//   delivered; it does not send anything itself. Real APNs/FCM delivery
//   requires Apple/Google credentials that are not available in this
//   environment (see BACKEND_BACKLOG.md) — this is a documented, deliberate
//   scope boundary, matching the existing browser web-push worker's
//   registration/delivery split (`notifications.web_push_subscriptions` and
//   `backend/apps/worker`).
//
// Specification:
//   QAS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::user::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushPlatform {
    Ios,
    Android,
}

impl PushPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

impl std::str::FromStr for PushPlatform {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ios" => Ok(Self::Ios),
            "android" => Ok(Self::Android),
            other => Err(DomainError::validation(format!(
                "unknown push platform: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for PushPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevicePushToken {
    pub id: Uuid,
    pub user_id: UserId,
    pub platform: PushPlatform,
    pub device_token: String,
    pub created_at: DateTime<Utc>,
}

impl DevicePushToken {
    pub fn register(
        user_id: UserId,
        platform: PushPlatform,
        device_token: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let device_token = device_token.into();
        if device_token.trim().is_empty() {
            return Err(DomainError::validation("device token is required"));
        }
        if device_token.len() > 4_096 {
            return Err(DomainError::validation("device token is too long"));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            user_id,
            platform,
            device_token,
            created_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_string_round_trip() {
        for variant in [PushPlatform::Ios, PushPlatform::Android] {
            assert_eq!(variant.as_str().parse::<PushPlatform>(), Ok(variant));
        }
    }

    #[test]
    fn rejects_blank_device_token() {
        let err = DevicePushToken::register(UserId::new(), PushPlatform::Ios, "  ", Utc::now())
            .unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn registers_a_valid_token() {
        let token =
            DevicePushToken::register(UserId::new(), PushPlatform::Android, "abc123", Utc::now())
                .expect("valid token");
        assert_eq!(token.device_token, "abc123");
        assert_eq!(token.platform, PushPlatform::Android);
    }
}
