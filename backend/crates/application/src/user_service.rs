// =============================================================================
// File:           backend/crates/application/src/user_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   User use cases: registration, lookup, status management, role changes.
//
// Specification:
//   QAS-000002, QAS-000003, QAS-000004, QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{User, UserId, UserRepository, UserRole};

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct CreateUserInput {
    pub email: String,
    pub display_name: String,
    pub role: UserRole,
}

pub struct UserService<R>
where
    R: UserRepository,
{
    users: R,
}

impl<R> UserService<R>
where
    R: UserRepository,
{
    pub fn new(users: R) -> Self {
        Self { users }
    }

    pub async fn register(&self, input: CreateUserInput) -> Result<User, ApplicationError> {
        // Check for duplicate email.
        if self.users.find_by_email(&input.email).await?.is_some() {
            return Err(ApplicationError::Conflict(format!(
                "a user with email '{}' already exists",
                input.email
            )));
        }
        let user = User::create(
            UserId::new(),
            input.email,
            input.display_name,
            input.role,
            Utc::now(),
        )?;
        self.users.create(&user).await?;
        Ok(user)
    }

    pub async fn get(&self, id: UserId) -> Result<User, ApplicationError> {
        self.users
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn get_by_email(&self, email: &str) -> Result<User, ApplicationError> {
        self.users
            .find_by_email(email)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn suspend(&self, id: UserId) -> Result<User, ApplicationError> {
        let mut user = self.get(id).await?;
        user.suspend()?;
        self.users.update(&user).await?;
        Ok(user)
    }

    pub async fn reactivate(&self, id: UserId) -> Result<User, ApplicationError> {
        let mut user = self.get(id).await?;
        user.reactivate()?;
        self.users.update(&user).await?;
        Ok(user)
    }

    pub async fn change_role(
        &self,
        id: UserId,
        new_role: UserRole,
    ) -> Result<User, ApplicationError> {
        let mut user = self.get(id).await?;
        user.change_role(new_role);
        self.users.update(&user).await?;
        Ok(user)
    }

    /// Sets or replaces the phone number used for OTP login. Required
    /// before a user can request an OTP challenge, since `OtpService`
    /// resolves accounts strictly by phone number.
    pub async fn set_phone(&self, id: UserId, phone: String) -> Result<User, ApplicationError> {
        if phone.trim().is_empty() {
            return Err(ApplicationError::Conflict(
                "phone number is required".into(),
            ));
        }
        if let Some(existing) = self.users.find_by_phone(&phone).await? {
            if existing.id != id {
                return Err(ApplicationError::Conflict(
                    "phone number is already linked to another account".into(),
                ));
            }
        }
        let mut user = self.get(id).await?;
        user.set_phone(phone);
        self.users.update(&user).await?;
        Ok(user)
    }
}
