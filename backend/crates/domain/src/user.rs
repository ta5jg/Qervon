// =============================================================================
// File:           backend/crates/domain/src/user.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Identity domain: user, role, and authentication entities.
//
// Specification:
//   QAS-000002, QAS-000003, QAS-000004, QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

// ---------------------------------------------------------------------------
// UserId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// UserRole — maps to permissions.yaml roles
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Customer,
    Company,
    Courier,
    Admin,
    SuperAdmin,
    Operator,
    Dispatcher,
    FleetManager,
    Support,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Customer => "customer",
            Self::Company => "company",
            Self::Courier => "courier",
            Self::Admin => "admin",
            Self::SuperAdmin => "super_admin",
            Self::Operator => "operator",
            Self::Dispatcher => "dispatcher",
            Self::FleetManager => "fleet_manager",
            Self::Support => "support",
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin | Self::SuperAdmin)
    }
}

impl std::str::FromStr for UserRole {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "customer" => Ok(Self::Customer),
            "company" => Ok(Self::Company),
            "courier" => Ok(Self::Courier),
            "admin" => Ok(Self::Admin),
            "super_admin" => Ok(Self::SuperAdmin),
            "operator" => Ok(Self::Operator),
            "dispatcher" => Ok(Self::Dispatcher),
            "fleet_manager" => Ok(Self::FleetManager),
            "support" => Ok(Self::Support),
            other => Err(DomainError::validation(format!(
                "unknown user role: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// UserStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Deleted => "deleted",
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "deleted" => Ok(Self::Deleted),
            other => Err(DomainError::validation(format!(
                "unknown user status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// User entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub phone: Option<String>,
    pub display_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn create(
        id: UserId,
        email: impl Into<String>,
        display_name: impl Into<String>,
        role: UserRole,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let email = email.into();
        let display_name = display_name.into();
        if email.trim().is_empty() || !email.contains('@') {
            return Err(DomainError::validation("a valid email is required"));
        }
        if display_name.trim().is_empty() {
            return Err(DomainError::validation("display name is required"));
        }
        Ok(Self {
            id,
            email,
            phone: None,
            display_name,
            role,
            status: UserStatus::Active,
            created_at: now,
        })
    }

    pub fn set_phone(&mut self, phone: impl Into<String>) {
        self.phone = Some(phone.into());
    }

    pub fn suspend(&mut self) -> Result<(), DomainError> {
        if self.status != UserStatus::Active {
            return Err(DomainError::invalid_transition(format!(
                "cannot suspend a {} user",
                self.status
            )));
        }
        self.status = UserStatus::Suspended;
        Ok(())
    }

    pub fn reactivate(&mut self) -> Result<(), DomainError> {
        if self.status != UserStatus::Suspended {
            return Err(DomainError::invalid_transition(format!(
                "can only reactivate a suspended user, current status: {}",
                self.status
            )));
        }
        self.status = UserStatus::Active;
        Ok(())
    }

    pub fn soft_delete(&mut self) -> Result<(), DomainError> {
        if self.status == UserStatus::Deleted {
            return Err(DomainError::invalid_transition("user is already deleted"));
        }
        self.status = UserStatus::Deleted;
        Ok(())
    }

    pub fn change_role(&mut self, new_role: UserRole) {
        self.role = new_role;
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user() -> User {
        User::create(
            UserId::new(),
            "test@qervon.com",
            "Test User",
            UserRole::Customer,
            Utc::now(),
        )
        .expect("valid user")
    }

    #[test]
    fn user_starts_active() {
        let u = sample_user();
        assert!(u.is_active());
        assert_eq!(u.role, UserRole::Customer);
    }

    #[test]
    fn rejects_empty_email() {
        let result = User::create(UserId::new(), "", "Name", UserRole::Customer, Utc::now());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_email_without_at() {
        let result = User::create(
            UserId::new(),
            "bademail",
            "Name",
            UserRole::Customer,
            Utc::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn suspend_and_reactivate() {
        let mut u = sample_user();
        u.suspend().expect("suspend");
        assert_eq!(u.status, UserStatus::Suspended);
        assert!(!u.is_active());

        u.reactivate().expect("reactivate");
        assert!(u.is_active());
    }

    #[test]
    fn deleted_user_cannot_be_deleted_again() {
        let mut u = sample_user();
        u.soft_delete().expect("delete");
        let err = u.soft_delete().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn change_role() {
        let mut u = sample_user();
        u.change_role(UserRole::Admin);
        assert!(u.role.is_admin());
    }

    #[test]
    fn role_round_trip() {
        assert_eq!(
            "super_admin".parse::<UserRole>().unwrap(),
            UserRole::SuperAdmin
        );
        assert_eq!(UserRole::FleetManager.as_str(), "fleet_manager");
    }
}
