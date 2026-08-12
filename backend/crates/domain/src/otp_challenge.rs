// =============================================================================
// File:           backend/crates/domain/src/otp_challenge.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   One-time-password (OTP) challenge domain model for phone-based login.
//   The challenge stores only an opaque hash of the code (computed by the
//   application layer); the domain never handles raw codes or hashing
//   algorithms, matching the existing Credential/RefreshSession split.
//
// Specification:
//   QAS-000002, QAS-000004, QLS-000001, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::tenant::TenantId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtpChallenge {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub phone: String,
    pub code_hash: String,
    pub attempts: u8,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

impl OtpChallenge {
    /// Maximum number of verification attempts allowed per challenge before
    /// it is permanently rejected, independent of expiry.
    pub const MAX_ATTEMPTS: u8 = 5;

    pub fn issue(
        tenant_id: TenantId,
        phone: impl Into<String>,
        code_hash: impl Into<String>,
        now: DateTime<Utc>,
        ttl: Duration,
    ) -> Result<Self, DomainError> {
        let phone = phone.into();
        if phone.trim().is_empty() {
            return Err(DomainError::validation("phone number is required"));
        }
        let code_hash = code_hash.into();
        if code_hash.trim().is_empty() {
            return Err(DomainError::validation("code hash is required"));
        }
        if ttl <= Duration::zero() {
            return Err(DomainError::validation("otp lifetime must be positive"));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            tenant_id,
            phone,
            code_hash,
            attempts: 0,
            created_at: now,
            expires_at: now + ttl,
            consumed_at: None,
        })
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed_at.is_some()
    }

    /// Verifies a caller-supplied hash of the entered code against the
    /// stored hash. Every call (successful or not) consumes one attempt,
    /// except when the challenge is already closed (consumed/expired), so a
    /// caller cannot retry a dead challenge indefinitely. On success the
    /// challenge is marked consumed and can never be verified again.
    pub fn verify(
        &mut self,
        provided_code_hash: &str,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if self.is_consumed() {
            return Err(DomainError::invalid_transition(
                "otp challenge already consumed",
            ));
        }
        if self.is_expired(now) {
            return Err(DomainError::invalid_transition("otp challenge expired"));
        }
        if self.attempts >= Self::MAX_ATTEMPTS {
            return Err(DomainError::invalid_transition(
                "otp verification attempt limit exceeded",
            ));
        }
        self.attempts += 1;
        if self.code_hash != provided_code_hash {
            return Err(DomainError::validation("invalid otp code"));
        }
        self.consumed_at = Some(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> TenantId {
        TenantId::new()
    }

    #[test]
    fn issuing_rejects_blank_phone() {
        let err = OtpChallenge::issue(tenant(), "  ", "hash", Utc::now(), Duration::minutes(5))
            .unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn correct_code_consumes_the_challenge() {
        let now = Utc::now();
        let mut challenge = OtpChallenge::issue(
            tenant(),
            "+905551112233",
            "correct-hash",
            now,
            Duration::minutes(5),
        )
        .expect("valid challenge");
        challenge.verify("correct-hash", now).expect("verifies");
        assert!(challenge.is_consumed());
        assert_eq!(challenge.attempts, 1);
    }

    #[test]
    fn wrong_code_is_rejected_and_consumes_an_attempt() {
        let now = Utc::now();
        let mut challenge = OtpChallenge::issue(
            tenant(),
            "+905551112233",
            "correct-hash",
            now,
            Duration::minutes(5),
        )
        .expect("valid challenge");
        let err = challenge.verify("wrong-hash", now).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
        assert!(!challenge.is_consumed());
        assert_eq!(challenge.attempts, 1);
    }

    #[test]
    fn expired_challenge_cannot_be_verified() {
        let now = Utc::now();
        let mut challenge = OtpChallenge::issue(
            tenant(),
            "+905551112233",
            "correct-hash",
            now,
            Duration::minutes(5),
        )
        .expect("valid challenge");
        let later = now + Duration::minutes(6);
        let err = challenge.verify("correct-hash", later).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn consumed_challenge_cannot_be_verified_again() {
        let now = Utc::now();
        let mut challenge = OtpChallenge::issue(
            tenant(),
            "+905551112233",
            "correct-hash",
            now,
            Duration::minutes(5),
        )
        .expect("valid challenge");
        challenge.verify("correct-hash", now).expect("first verify");
        let err = challenge.verify("correct-hash", now).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn attempt_limit_locks_out_the_challenge() {
        let now = Utc::now();
        let mut challenge = OtpChallenge::issue(
            tenant(),
            "+905551112233",
            "correct-hash",
            now,
            Duration::minutes(5),
        )
        .expect("valid challenge");
        for _ in 0..OtpChallenge::MAX_ATTEMPTS {
            let _ = challenge.verify("wrong-hash", now);
        }
        let err = challenge.verify("correct-hash", now).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }
}
