// =============================================================================
// File:           backend/crates/application/src/otp_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   OTP (one-time-password) phone login use cases: issuing and verifying a
//   challenge for an existing user's phone number.
//
//   Known limitation (see BACKEND_BACKLOG.md): no real SMS provider is wired
//   yet. `request_otp` returns the raw code to the caller so the API layer
//   can log it for local development; production SMS delivery requires
//   integrating a provider (e.g. Twilio) once credentials are available.
//
// Specification:
//   QAS-000002, QAS-000004, QLS-000001, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{Duration, Utc};
use qervon_domain::{OtpChallenge, OtpChallengeRepository, TenantId, User, UserRepository};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::ApplicationError;

/// How long an issued OTP code remains valid.
pub const OTP_TTL: Duration = Duration::minutes(5);

pub struct OtpService<UR, OR>
where
    UR: UserRepository,
    OR: OtpChallengeRepository,
{
    users: UR,
    challenges: OR,
}

impl<UR, OR> OtpService<UR, OR>
where
    UR: UserRepository,
    OR: OtpChallengeRepository,
{
    pub fn new(users: UR, challenges: OR) -> Self {
        Self { users, challenges }
    }

    /// Issues a new OTP challenge for a phone number that already belongs to
    /// an active user. Returns the raw numeric code (not persisted in plain
    /// text) so the caller can deliver it; this service never sends SMS.
    pub async fn request_otp(
        &self,
        tenant_id: TenantId,
        phone: &str,
    ) -> Result<String, ApplicationError> {
        let user = self
            .users
            .find_by_phone(phone)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        if !user.is_active() {
            return Err(ApplicationError::Conflict("user is not active".into()));
        }
        let code = generate_numeric_code();
        let challenge = OtpChallenge::issue(
            tenant_id,
            phone.to_string(),
            hash_code(&code),
            Utc::now(),
            OTP_TTL,
        )?;
        self.challenges.create(&challenge).await?;
        Ok(code)
    }

    /// Verifies a submitted code against the latest active challenge for
    /// this tenant+phone pair and returns the matching user on success.
    pub async fn verify_otp(
        &self,
        tenant_id: TenantId,
        phone: &str,
        code: &str,
    ) -> Result<User, ApplicationError> {
        let now = Utc::now();
        let mut challenge = self
            .challenges
            .find_latest_active(tenant_id, phone, now)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        challenge.verify(&hash_code(code), now)?;
        self.challenges.update(&challenge).await?;
        self.users
            .find_by_phone(phone)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}

/// Generates a cryptographically random 6-digit numeric code, zero-padded.
fn generate_numeric_code() -> String {
    let value: u32 = rand::rng().random_range(0..1_000_000);
    format!("{value:06}")
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qervon_domain::{UserId, UserRole};
    use qervon_infrastructure::memory::InMemoryStore;

    fn users_and_challenges() -> (
        impl UserRepository,
        impl OtpChallengeRepository,
        qervon_infrastructure::memory::InMemoryStore,
    ) {
        let store = InMemoryStore::new();
        (
            store.user_repository(),
            store.otp_challenge_repository(),
            store,
        )
    }

    #[tokio::test]
    async fn requesting_otp_for_unknown_phone_fails() {
        let (users, challenges, _store) = users_and_challenges();
        let service = OtpService::new(users, challenges);
        let err = service
            .request_otp(TenantId::new(), "+905551112233")
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn full_request_and_verify_round_trip() {
        let (users, challenges, _store) = users_and_challenges();
        let mut user = User::create(
            UserId::new(),
            "courier@qervon.test",
            "Courier",
            UserRole::Courier,
            Utc::now(),
        )
        .expect("valid user");
        user.set_phone("+905551112233");
        users.create(&user).await.expect("create user");

        let service = OtpService::new(users, challenges);
        let tenant_id = TenantId::new();
        let code = service
            .request_otp(tenant_id, "+905551112233")
            .await
            .expect("request otp");
        assert_eq!(code.len(), 6);

        let verified = service
            .verify_otp(tenant_id, "+905551112233", &code)
            .await
            .expect("verify otp");
        assert_eq!(verified.id, user.id);
    }

    #[tokio::test]
    async fn wrong_code_is_rejected() {
        let (users, challenges, _store) = users_and_challenges();
        let mut user = User::create(
            UserId::new(),
            "courier2@qervon.test",
            "Courier",
            UserRole::Courier,
            Utc::now(),
        )
        .expect("valid user");
        user.set_phone("+905551112244");
        users.create(&user).await.expect("create user");

        let service = OtpService::new(users, challenges);
        let tenant_id = TenantId::new();
        service
            .request_otp(tenant_id, "+905551112244")
            .await
            .expect("request otp");

        let err = service
            .verify_otp(tenant_id, "+905551112244", "000000")
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }
}
