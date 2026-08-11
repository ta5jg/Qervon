// =============================================================================
// File:           backend/crates/infrastructure/tests/postgres_repositories.rs
// Project:        Qervon
// Description:    Opt-in integration tests for durable PostgreSQL repositories.
// =============================================================================

use chrono::{Duration, NaiveDate, Utc};
use qervon_domain::{
    Courier, CourierPayout, CourierPayoutRepository, CourierRepository, CustomerId,
    CustomerProfile, CustomerRepository, Location, Money, SavedAddress, User, UserId,
    UserRepository, UserRole, Vehicle, VehicleId, VehicleRepository, VehicleType,
};
use qervon_infrastructure::{
    postgres::PgPoolOptions, PgCourierPayoutRepository, PgCourierRepository, PgCustomerRepository,
    PgUserRepository, PgVehicleRepository,
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
}
