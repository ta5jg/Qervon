// =============================================================================
// File:           backend/crates/infrastructure/tests/postgres_repositories.rs
// Project:        Qervon
// Description:    Opt-in integration tests for durable PostgreSQL repositories.
// =============================================================================

use chrono::{Duration, NaiveDate, Utc};
use qervon_domain::{
    ColdChainTelemetry, ColdChainTelemetryRepository, Courier, CourierPayout,
    CourierPayoutRepository, CourierRepository, CustomerId, CustomerProfile, CustomerRepository,
    FieldServiceAppointmentRepository, FieldServiceScheduler, Location, Money,
    RouteBreadcrumbRepository, SavedAddress, TenantId, TimeSlotWindow, User, UserId,
    UserRepository, UserRole, Vehicle, VehicleId, VehicleRepository, VehicleType, WarehouseHub,
    WarehouseHubRepository,
};
use qervon_infrastructure::{
    postgres::PgPoolOptions, PgColdChainTelemetryRepository, PgCourierPayoutRepository,
    PgCourierRepository, PgCustomerRepository, PgFieldServiceAppointmentRepository,
    PgRouteBreadcrumbRepository, PgUserRepository, PgVehicleRepository, PgWarehouseHubRepository,
};
use uuid::Uuid;

fn test_database_url() -> String {
    std::env::var("QERVON_TEST_DATABASE_URL")
        .expect("QERVON_TEST_DATABASE_URL is required; run scripts/test-postgres-integration.sh")
}

#[tokio::test]
#[ignore = "requires an explicitly configured disposable PostgreSQL database"]
async fn postgres_repositories_round_trip() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_database_url())
        .await
        .expect("connect to Qervon test database");
    let now = Utc::now();
    let suffix = Uuid::now_v7();

    let users = PgUserRepository::new(pool.clone());
    let customers = PgCustomerRepository::new(pool.clone());
    let couriers = PgCourierRepository::new(pool.clone());
    let vehicles = PgVehicleRepository::new(pool.clone());
    let payouts = PgCourierPayoutRepository::new(pool.clone());

    let user = User::create(
        UserId::new(),
        format!("customer-{suffix}@qervon.test"),
        "PostgreSQL Customer",
        UserRole::Customer,
        now,
    )
    .expect("valid customer user");
    users.create(&user).await.expect("persist customer user");

    let mut profile = CustomerProfile::create(CustomerId::new(), user.id, now);
    profile.set_company("Qervon Test Ltd.", "TR-TEST-001");
    profile.add_address(
        SavedAddress::new(
            "Merkez",
            Location::new(41.0082, 28.9784).expect("valid location"),
            "Sultanahmet, İstanbul",
        )
        .expect("valid address"),
    );
    profile.add_loyalty_points(120);
    customers.create(&profile).await.expect("persist profile");

    let loaded_profile = customers
        .find_by_user(user.id)
        .await
        .expect("read profile")
        .expect("profile exists");
    assert_eq!(loaded_profile.id, profile.id);
    assert_eq!(loaded_profile.loyalty_points, 120);
    assert_eq!(loaded_profile.addresses.len(), 1);
    assert!(loaded_profile.default_address().is_some());

    let courier = Courier::create(
        Uuid::now_v7(),
        "PostgreSQL Courier",
        VehicleType::Motorcycle,
        now,
    )
    .expect("valid courier");
    couriers.create(&courier).await.expect("persist courier");

    let mut vehicle = Vehicle::register(
        VehicleId::new(),
        format!("34 QV {}", &suffix.to_string()[..6]),
        VehicleType::Motorcycle,
        Some(NaiveDate::from_ymd_opt(2028, 1, 1).expect("valid date")),
        now,
    )
    .expect("valid vehicle");
    vehicle.assign_courier(courier.id).expect("assign courier");
    vehicles.create(&vehicle).await.expect("persist vehicle");

    let loaded_vehicle = vehicles
        .find_by_plate(&vehicle.plate_number.to_lowercase())
        .await
        .expect("read vehicle")
        .expect("vehicle exists");
    assert_eq!(loaded_vehicle.assigned_courier_id, Some(courier.id));
    assert!(vehicles
        .list_active()
        .await
        .expect("list vehicles")
        .iter()
        .any(|item| item.id == vehicle.id));

    let mut payout = CourierPayout::create(
        courier.id,
        now - Duration::days(7),
        now,
        Money::new(10_000, "TRY").expect("gross amount"),
        Money::new(1_500, "TRY").expect("commission"),
        now,
    )
    .expect("valid payout");
    payouts.create(&payout).await.expect("persist payout");
    payout.approve().expect("approve payout");
    payouts.update(&payout).await.expect("update payout");

    let persisted_payout = payouts
        .find_by_courier(courier.id)
        .await
        .expect("read payouts")
        .pop()
        .expect("payout exists");
    assert_eq!(persisted_payout.net_amount.amount_minor, 8_500);
    assert_eq!(persisted_payout.status.as_str(), "approved");

    // --- LOS campaign domain expansion: warehouse, cold-chain, field
    // service, and route-history breadcrumbs. These exercise the raw SQL in
    // postgres.rs directly against a real schema, since the HTTP-level
    // tenant-isolation tests in api_flow.rs only run against InMemoryStore.
    let tenant_id = TenantId::new();

    let warehouse = PgWarehouseHubRepository::new(pool.clone());
    let hub = WarehouseHub::new(
        tenant_id,
        format!("HUB-{}", &suffix.to_string()[..8]),
        "PostgreSQL Test Hub",
        Location::new(41.02, 28.95).expect("valid hub location"),
        500,
    );
    warehouse.create_hub(&hub).await.expect("persist hub");
    let mut loaded_hub = warehouse
        .find_hub_by_id(hub.id)
        .await
        .expect("read hub")
        .expect("hub exists");
    assert_eq!(loaded_hub.tenant_id, tenant_id);
    assert_eq!(loaded_hub.active_parcels, 0);

    loaded_hub.receive_parcels(15).expect("receive parcels");
    warehouse.update_hub(&loaded_hub).await.expect("update hub");
    let refreshed_hub = warehouse
        .find_hub_by_id(hub.id)
        .await
        .expect("read hub again")
        .expect("hub exists");
    assert_eq!(refreshed_hub.active_parcels, 15);
    assert!(warehouse
        .list_hubs_for_tenant(tenant_id)
        .await
        .expect("list hubs")
        .iter()
        .any(|item| item.id == hub.id));

    let manifest = refreshed_hub
        .clone()
        .dispatch_manifest(courier.id, vec![])
        .expect("dispatch manifest");
    warehouse
        .create_manifest(&manifest)
        .await
        .expect("persist manifest");

    let cold_chain = PgColdChainTelemetryRepository::new(pool.clone());
    let telemetry =
        ColdChainTelemetry::new(tenant_id, Uuid::now_v7(), "SENS-PG-1", 11.0, 40.0, 2.0, 8.0);
    cold_chain
        .create(&telemetry)
        .await
        .expect("persist telemetry");
    let telemetry_for_tenant = cold_chain
        .list_for_tenant(tenant_id, Some(telemetry.order_id))
        .await
        .expect("list telemetry");
    assert_eq!(telemetry_for_tenant.len(), 1);
    assert!(telemetry_for_tenant[0].is_violation);

    let field_service = PgFieldServiceAppointmentRepository::new(pool.clone());
    let appointment = FieldServiceScheduler::schedule_appointment(
        tenant_id,
        Uuid::now_v7(),
        "PostgreSQL Bakım",
        "2026-09-01",
        TimeSlotWindow::Afternoon,
    );
    field_service
        .create(&appointment)
        .await
        .expect("persist appointment");
    let appointments_for_tenant = field_service
        .list_for_tenant(tenant_id)
        .await
        .expect("list appointments");
    assert!(appointments_for_tenant
        .iter()
        .any(|item| item.id == appointment.id));

    let route_history = PgRouteBreadcrumbRepository::new(pool.clone());
    let breadcrumb = qervon_domain::RouteBreadcrumb {
        id: Uuid::now_v7(),
        tenant_id,
        courier_id: courier.id,
        location: Location::new(41.03, 28.96).expect("valid breadcrumb location"),
        speed_kmh: 28.0,
        battery_level: 77,
        timestamp: now,
    };
    route_history
        .create(&breadcrumb)
        .await
        .expect("persist breadcrumb");
    let day = now.format("%Y-%m-%d").to_string();
    let breadcrumbs_for_day = route_history
        .list_for_courier_and_date(tenant_id, courier.id, &day)
        .await
        .expect("list breadcrumbs");
    assert_eq!(breadcrumbs_for_day.len(), 1);
    assert_eq!(breadcrumbs_for_day[0].id, breadcrumb.id);
}
