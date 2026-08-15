// =============================================================================
// File:           backend/crates/application/src/device_push_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Native mobile push device registration use cases: idempotent
//   registration, listing, and removal, scoped to the signed-in user.
//
// Specification:
//   QAS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{AppVariant, DevicePushToken, DevicePushTokenRepository, PushPlatform, UserId};
use uuid::Uuid;

use crate::error::ApplicationError;

pub struct DevicePushService<R>
where
    R: DevicePushTokenRepository,
{
    tokens: R,
}

impl<R> DevicePushService<R>
where
    R: DevicePushTokenRepository,
{
    pub fn new(tokens: R) -> Self {
        Self { tokens }
    }

    /// Registers a device token for push delivery. Re-registering the same
    /// (user, device_token) pair is a no-op that returns the existing
    /// registration, so app-launch registration calls stay idempotent.
    pub async fn register(
        &self,
        user_id: UserId,
        platform: PushPlatform,
        app_variant: AppVariant,
        device_token: String,
    ) -> Result<DevicePushToken, ApplicationError> {
        if let Some(existing) = self
            .tokens
            .find_by_user_and_token(user_id, &device_token)
            .await?
        {
            return Ok(existing);
        }
        let token =
            DevicePushToken::register(user_id, platform, app_variant, device_token, Utc::now())?;
        self.tokens.create(&token).await?;
        Ok(token)
    }

    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DevicePushToken>, ApplicationError> {
        Ok(self.tokens.list_for_user(user_id).await?)
    }

    pub async fn unregister(&self, user_id: UserId, id: Uuid) -> Result<(), ApplicationError> {
        self.tokens.delete(user_id, id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qervon_infrastructure::memory::InMemoryStore;

    #[tokio::test]
    async fn registering_the_same_device_twice_is_idempotent() {
        let store = InMemoryStore::new();
        let service = DevicePushService::new(store.device_push_token_repository());
        let user_id = UserId::new();

        let first = service
            .register(
                user_id,
                PushPlatform::Ios,
                AppVariant::Courier,
                "device-abc".into(),
            )
            .await
            .expect("first registration");
        let second = service
            .register(
                user_id,
                PushPlatform::Ios,
                AppVariant::Courier,
                "device-abc".into(),
            )
            .await
            .expect("second registration");
        assert_eq!(first.id, second.id);

        let tokens = service.list_for_user(user_id).await.expect("list tokens");
        assert_eq!(tokens.len(), 1);
    }

    #[tokio::test]
    async fn unregistering_someone_elses_token_is_a_silent_no_op() {
        let store = InMemoryStore::new();
        let service = DevicePushService::new(store.device_push_token_repository());
        let owner = UserId::new();
        let stranger = UserId::new();

        let token = service
            .register(
                owner,
                PushPlatform::Android,
                AppVariant::Customer,
                "device-xyz".into(),
            )
            .await
            .expect("register");

        service
            .unregister(stranger, token.id)
            .await
            .expect("no-op delete");
        let tokens = service.list_for_user(owner).await.expect("list tokens");
        assert_eq!(tokens.len(), 1, "stranger must not be able to delete it");

        service
            .unregister(owner, token.id)
            .await
            .expect("owner deletes");
        let tokens = service.list_for_user(owner).await.expect("list tokens");
        assert!(tokens.is_empty());
    }
}
