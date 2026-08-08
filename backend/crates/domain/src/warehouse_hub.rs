// =============================================================================
// File:           backend/crates/domain/src/warehouse_hub.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Warehouse, Cross-Docking Hub & Parcel Manifest Assignment Domain Model.
//
// Specification:
//   QAS-000001, QES-000002.
// =============================================================================

use crate::Location;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseHub {
    pub id: uuid::Uuid,
    pub hub_code: String, // e.g. "HUB-IST-01"
    pub hub_name: String,
    pub location: Location,
    pub capacity_parcels: u32,
    pub active_parcels: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubManifestAssignment {
    pub id: uuid::Uuid,
    pub hub_id: uuid::Uuid,
    pub courier_id: uuid::Uuid,
    pub order_ids: Vec<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl WarehouseHub {
    pub fn new(hub_code: impl Into<String>, hub_name: impl Into<String>, location: Location, capacity: u32) -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
            hub_code: hub_code.into(),
            hub_name: hub_name.into(),
            location,
            capacity_parcels: capacity,
            active_parcels: 0,
        }
    }

    pub fn receive_parcels(&mut self, count: u32) -> Result<(), String> {
        if self.active_parcels + count > self.capacity_parcels {
            return Err("Warehouse capacity exceeded".into());
        }
        self.active_parcels += count;
        Ok(())
    }

    pub fn dispatch_manifest(&mut self, courier_id: uuid::Uuid, order_ids: Vec<uuid::Uuid>) -> Result<HubManifestAssignment, String> {
        let count = order_ids.len() as u32;
        if self.active_parcels < count {
            return Err("Not enough parcels in hub".into());
        }
        self.active_parcels -= count;

        Ok(HubManifestAssignment {
            id: uuid::Uuid::now_v7(),
            hub_id: self.id,
            courier_id,
            order_ids,
            created_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receives_and_dispatches_parcels_manifest() {
        let loc = Location::new(41.06, 28.93).unwrap();
        let mut hub = WarehouseHub::new("HUB-01", "İstanbul Ana Transfer Merkezi", loc, 1000);

        hub.receive_parcels(50).unwrap();
        assert_eq!(hub.active_parcels, 50);

        let courier_id = uuid::Uuid::now_v7();
        let orders = vec![uuid::Uuid::now_v7(), uuid::Uuid::now_v7()];
        let manifest = hub.dispatch_manifest(courier_id, orders).unwrap();

        assert_eq!(manifest.order_ids.len(), 2);
        assert_eq!(hub.active_parcels, 48);
    }
}
