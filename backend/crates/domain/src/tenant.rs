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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub uuid::Uuid);

impl TenantId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub uuid::Uuid);

impl BranchId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCompany {
    pub id: TenantId,
    pub company_name: String,
    pub category: String, // e.g. "Restaurant", "Pharmacy", "E-Commerce", "Logistics"
    pub created_at: chrono::DateTime<chrono::Utc>,
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
