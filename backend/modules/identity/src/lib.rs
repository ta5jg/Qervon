// =============================================================================
// File:           backend/modules/identity/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Identity domain module: public boundary over user and auth use cases.
//
// Specification:
//   QAS-000002, QAS-000003, QAS-000004, QAS-000005, QAS-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{CreateUserInput, UserService};
use qervon_domain::{User, UserId, UserRepository, UserRole};

pub struct IdentityModule<R>
where
    R: UserRepository,
{
    service: UserService<R>,
}

impl<R> IdentityModule<R>
where
    R: UserRepository,
{
    pub fn new(users: R) -> Self {
        Self {
            service: UserService::new(users),
        }
    }

    pub async fn register_user(
        &self,
        input: CreateUserInput,
    ) -> Result<User, qervon_application::ApplicationError> {
        self.service.register(input).await
    }

    pub async fn get_user(
        &self,
        id: UserId,
    ) -> Result<User, qervon_application::ApplicationError> {
        self.service.get(id).await
    }

    pub async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<User, qervon_application::ApplicationError> {
        self.service.get_by_email(email).await
    }

    pub async fn suspend_user(
        &self,
        id: UserId,
    ) -> Result<User, qervon_application::ApplicationError> {
        self.service.suspend(id).await
    }

    pub async fn reactivate_user(
        &self,
        id: UserId,
    ) -> Result<User, qervon_application::ApplicationError> {
        self.service.reactivate(id).await
    }

    pub async fn change_user_role(
        &self,
        id: UserId,
        new_role: UserRole,
    ) -> Result<User, qervon_application::ApplicationError> {
        self.service.change_role(id, new_role).await
    }
}
