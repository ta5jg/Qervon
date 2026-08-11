//! One-time, VPS-local bootstrap for the first Qervon tenant owner.
//! This binary intentionally exposes no HTTP surface and requires an explicit
//! confirmation variable so it cannot run accidentally from a shell profile.

use chrono::Utc;
use qervon_application::AuthService;
use qervon_domain::{
    TenantCompany, TenantId, TenantMemberRole, TenantMembership, TenantRepository, UserRepository,
    UserRole,
};
use qervon_infrastructure::postgres::{
    PgCredentialRepository, PgPoolOptions, PgTenantRepository, PgUserRepository,
};

fn required(name: &str) -> Result<String, std::io::Error> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{name} is required"),
            )
        })
}

fn valid_slug(slug: &str) -> bool {
    (3..=63).contains(&slug.len())
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("QERVON_BOOTSTRAP_ALLOW").as_deref() != Ok("confirm") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "set QERVON_BOOTSTRAP_ALLOW=confirm to create the first tenant owner",
        )
        .into());
    }
    let database_url = required("DATABASE_URL")?;
    let tenant_name = required("QERVON_BOOTSTRAP_TENANT_NAME")?;
    let tenant_slug = required("QERVON_BOOTSTRAP_TENANT_SLUG")?;
    let email = required("QERVON_BOOTSTRAP_EMAIL")?;
    let password = required("QERVON_BOOTSTRAP_PASSWORD")?;
    if !valid_slug(&tenant_slug) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "QERVON_BOOTSTRAP_TENANT_SLUG must be 3-63 lowercase letters, digits, or hyphens",
        )
        .into());
    }
    if password.len() < 12 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "QERVON_BOOTSTRAP_PASSWORD must contain at least 12 characters",
        )
        .into());
    }

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;
    let users = PgUserRepository::new(pool.clone());
    let credentials = PgCredentialRepository::new(pool.clone());
    let tenants = PgTenantRepository::new(pool);
    if users.find_by_email(&email).await?.is_some()
        || tenants.find_by_slug(&tenant_slug).await?.is_some()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "bootstrap refused: the requested email or tenant slug already exists",
        )
        .into());
    }

    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: tenant_name,
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    tenants.create_tenant(&tenant, &tenant_slug).await?;
    let auth = AuthService::new(users, credentials);
    let user = auth
        .register(
            email,
            "Initial Tenant Owner".into(),
            password,
            UserRole::SuperAdmin,
        )
        .await?;
    tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: user.id,
            role: TenantMemberRole::Owner,
            joined_at: Utc::now(),
        })
        .await?;

    println!("Bootstrap complete. Tenant owner created for slug: {tenant_slug}");
    Ok(())
}
