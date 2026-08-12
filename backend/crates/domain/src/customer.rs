// =============================================================================
// File:           backend/crates/domain/src/customer.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Customer domain: profiles, addresses, and favorites.
//
// Specification:
//   QLS-000005, QAS-000002, QAS-000004, QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::location::Location;
use crate::user::UserId;

// ---------------------------------------------------------------------------
// CustomerId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomerId(pub Uuid);

impl CustomerId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for CustomerId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SavedAddress — customer's address book entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedAddress {
    pub id: Uuid,
    pub label: String,
    pub location: Location,
    pub full_address: String,
    pub is_default: bool,
}

impl SavedAddress {
    pub fn new(
        label: impl Into<String>,
        location: Location,
        full_address: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let label = label.into();
        if label.trim().is_empty() {
            return Err(DomainError::validation("address label is required"));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            label,
            location,
            full_address: full_address.into(),
            is_default: false,
        })
    }
}

// ---------------------------------------------------------------------------
// CustomerProfile entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomerProfile {
    pub id: CustomerId,
    pub user_id: UserId,
    pub company_name: Option<String>,
    pub tax_id: Option<String>,
    pub addresses: Vec<SavedAddress>,
    pub loyalty_points: u64,
    pub created_at: DateTime<Utc>,
}

impl CustomerProfile {
    pub fn create(id: CustomerId, user_id: UserId, now: DateTime<Utc>) -> Self {
        Self {
            id,
            user_id,
            company_name: None,
            tax_id: None,
            addresses: Vec::new(),
            loyalty_points: 0,
            created_at: now,
        }
    }

    pub fn set_company(&mut self, name: impl Into<String>, tax_id: impl Into<String>) {
        self.company_name = Some(name.into());
        self.tax_id = Some(tax_id.into());
    }

    pub fn add_address(&mut self, address: SavedAddress) {
        if self.addresses.is_empty() {
            // First address becomes the default.
            let mut addr = address;
            addr.is_default = true;
            self.addresses.push(addr);
        } else {
            self.addresses.push(address);
        }
    }

    pub fn set_default_address(&mut self, address_id: Uuid) -> Result<(), DomainError> {
        let found = self.addresses.iter().any(|a| a.id == address_id);
        if !found {
            return Err(DomainError::NotFound(format!(
                "address {address_id} not found in customer profile"
            )));
        }
        for addr in &mut self.addresses {
            addr.is_default = addr.id == address_id;
        }
        Ok(())
    }

    pub fn remove_address(&mut self, address_id: Uuid) -> Result<(), DomainError> {
        let before = self.addresses.len();
        self.addresses.retain(|a| a.id != address_id);
        if self.addresses.len() == before {
            return Err(DomainError::NotFound(format!(
                "address {address_id} not found"
            )));
        }
        // If the removed address was the default, promote the first one.
        if !self.addresses.is_empty() && !self.addresses.iter().any(|a| a.is_default) {
            self.addresses[0].is_default = true;
        }
        Ok(())
    }

    pub fn default_address(&self) -> Option<&SavedAddress> {
        self.addresses.iter().find(|a| a.is_default)
    }

    pub fn add_loyalty_points(&mut self, points: u64) {
        self.loyalty_points = self.loyalty_points.saturating_add(points);
    }

    pub fn spend_loyalty_points(&mut self, points: u64) -> Result<(), DomainError> {
        if self.loyalty_points < points {
            return Err(DomainError::validation(format!(
                "insufficient loyalty points: have {}, need {}",
                self.loyalty_points, points
            )));
        }
        self.loyalty_points -= points;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Location;

    fn sample_profile() -> CustomerProfile {
        CustomerProfile::create(CustomerId::new(), UserId::new(), Utc::now())
    }

    fn istanbul_address() -> SavedAddress {
        SavedAddress::new(
            "Ev",
            Location::new(41.0082, 28.9784).unwrap(),
            "Sultanahmet, Fatih/İstanbul",
        )
        .unwrap()
    }

    #[test]
    fn profile_starts_empty() {
        let p = sample_profile();
        assert!(p.addresses.is_empty());
        assert_eq!(p.loyalty_points, 0);
        assert!(p.company_name.is_none());
    }

    #[test]
    fn first_address_becomes_default() {
        let mut p = sample_profile();
        p.add_address(istanbul_address());
        assert_eq!(p.addresses.len(), 1);
        assert!(p.addresses[0].is_default);
    }

    #[test]
    fn set_default_address() {
        let mut p = sample_profile();
        let addr1 = istanbul_address();
        let addr2 = SavedAddress::new(
            "İş",
            Location::new(41.0, 29.0).unwrap(),
            "Maslak, Sarıyer/İstanbul",
        )
        .unwrap();
        let addr2_id = addr2.id;

        p.add_address(addr1);
        p.add_address(addr2);
        p.set_default_address(addr2_id).expect("set default");

        assert_eq!(p.default_address().unwrap().id, addr2_id);
    }

    #[test]
    fn remove_address_promotes_first() {
        let mut p = sample_profile();
        let addr1 = istanbul_address();
        let addr1_id = addr1.id;
        let addr2 = SavedAddress::new("İş", Location::new(41.0, 29.0).unwrap(), "Maslak").unwrap();

        p.add_address(addr1);
        p.add_address(addr2);
        p.remove_address(addr1_id).expect("remove");

        assert_eq!(p.addresses.len(), 1);
        assert!(p.addresses[0].is_default);
    }

    #[test]
    fn loyalty_points_spend_and_earn() {
        let mut p = sample_profile();
        p.add_loyalty_points(100);
        assert_eq!(p.loyalty_points, 100);

        p.spend_loyalty_points(40).expect("spend");
        assert_eq!(p.loyalty_points, 60);

        let err = p.spend_loyalty_points(100).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn set_company_info() {
        let mut p = sample_profile();
        p.set_company("ACME Ltd.", "1234567890");
        assert_eq!(p.company_name.as_deref(), Some("ACME Ltd."));
        assert_eq!(p.tax_id.as_deref(), Some("1234567890"));
    }
}
