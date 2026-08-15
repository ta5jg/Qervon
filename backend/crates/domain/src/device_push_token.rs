// =============================================================================
// File:           backend/crates/domain/src/device_push_token.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Native mobile push device registration. Records a device token plus
//   which app variant (Courier vs Customer) issued it — required because
//   the two iOS apps have distinct bundle identifiers, and APNs rejects a
//   push whose `apns-topic` header does not exactly match the bundle id
//   that issued the token. Real Android/FCM delivery still requires
//   credentials that are not available in this environment (see
//   BACKEND_BACKLOG.md); iOS/APNs delivery is real as of 2026-08-16 (see
//   `backend/apps/api-gateway/src/apns.rs`).
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

/// Which native app issued this device token. Distinct bundle ids
/// (`com.qervon.ios.courier` vs `com.qervon.ios.customer`) mean a push
/// provider must pick the matching one, not a per-platform constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppVariant {
    Courier,
    Customer,
}

impl AppVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Courier => "courier",
            Self::Customer => "customer",
        }
    }
}

impl std::str::FromStr for AppVariant {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "courier" => Ok(Self::Courier),
            "customer" => Ok(Self::Customer),
            other => Err(DomainError::validation(format!(
                "unknown app variant: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for AppVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevicePushToken {
    pub id: Uuid,
    pub user_id: UserId,
    pub platform: PushPlatform,
    pub app_variant: AppVariant,
    pub device_token: String,
    pub created_at: DateTime<Utc>,
}

impl DevicePushToken {
    pub fn register(
        user_id: UserId,
        platform: PushPlatform,
        app_variant: AppVariant,
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
            app_variant,
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
    fn app_variant_string_round_trip() {
        for variant in [AppVariant::Courier, AppVariant::Customer] {
            assert_eq!(variant.as_str().parse::<AppVariant>(), Ok(variant));
        }
    }

    #[test]
    fn rejects_blank_device_token() {
        let err = DevicePushToken::register(
            UserId::new(),
            PushPlatform::Ios,
            AppVariant::Courier,
            "  ",
            Utc::now(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn registers_a_valid_token() {
        let token = DevicePushToken::register(
            UserId::new(),
            PushPlatform::Android,
            AppVariant::Customer,
            "abc123",
            Utc::now(),
        )
        .expect("valid token");
        assert_eq!(token.device_token, "abc123");
        assert_eq!(token.platform, PushPlatform::Android);
        assert_eq!(token.app_variant, AppVariant::Customer);
    }
}
