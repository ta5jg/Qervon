// =============================================================================
// File:           backend/crates/domain/src/tenant.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Multi-Tenant Company & Branch Isolation Domain Model.
//
// Specification:
//   QAS-000001, QAS-000005, QES-000002.
// =============================================================================

use crate::user::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub uuid::Uuid);

impl TenantId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub uuid::Uuid);

impl BranchId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl Default for BranchId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCompany {
    pub id: TenantId,
    pub company_name: String,
    pub category: String, // e.g. "Restaurant", "Pharmacy", "E-Commerce", "Logistics"
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// The tenant-scoped authority is intentionally separate from the global user
/// role. A user can operate in more than one tenant without becoming an admin
/// for every tenant in the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantMemberRole {
    Owner,
    Admin,
    Operator,
    Member,
}

impl TenantMemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Operator => "operator",
            Self::Member => "member",
        }
    }
}

impl std::str::FromStr for TenantMemberRole {
    type Err = crate::error::DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "operator" => Ok(Self::Operator),
            "member" => Ok(Self::Member),
            _ => Err(crate::error::DomainError::validation(
                "invalid tenant member role",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantMembership {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role: TenantMemberRole,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranch {
    pub id: BranchId,
    pub tenant_id: TenantId,
    pub branch_name: String,
    pub city: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_tenant_and_branch() {
        let tenant_id = TenantId::new();
        let branch_id = BranchId::new();

        let company = TenantCompany {
            id: tenant_id,
            company_name: "Lezzet Restoranları A.Ş.".into(),
            category: "Restaurant".into(),
            created_at: chrono::Utc::now(),
        };

        let branch = TenantBranch {
            id: branch_id,
            tenant_id,
            branch_name: "Kadıköy Şubesi".into(),
            city: "İstanbul".into(),
        };

        assert_eq!(branch.tenant_id, company.id);
    }
}
