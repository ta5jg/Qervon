use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use qervon_domain::UserRole;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub fn hash_password(password: &str) -> Result<String, &'static str> {
    if password.len() < 12 {
        return Err("password must contain at least 12 characters");
    }
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "could not hash password")
}

pub fn verify_password(password_hash: &str, password: &str) -> Result<(), &'static str> {
    let parsed = PasswordHash::new(password_hash).map_err(|_| "invalid stored password hash")?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| "invalid credentials")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessClaims {
    pub subject: Uuid,
    pub tenant_id: Uuid,
    pub role: UserRole,
    pub expires_at: i64,
}

pub fn issue_access_token(
    secret: &[u8],
    subject: Uuid,
    tenant_id: Uuid,
    role: UserRole,
    lifetime: Duration,
) -> Result<String, &'static str> {
    let claims = AccessClaims {
        subject,
        tenant_id,
        role,
        expires_at: (Utc::now() + lifetime).timestamp(),
    };
    let payload = serde_json::to_vec(&claims).map_err(|_| "could not encode access claims")?;
    let encoded = URL_SAFE_NO_PAD.encode(payload);
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| "invalid signing key")?;
    mac.update(encoded.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("qv1.{encoded}.{signature}"))
}

pub fn verify_access_token(secret: &[u8], token: &str) -> Result<AccessClaims, &'static str> {
    let mut parts = token.split('.');
    let (Some(version), Some(encoded), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err("malformed access token");
    };
    if version != "qv1" {
        return Err("unsupported access token version");
    }
    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| "malformed access token")?;
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| "invalid signing key")?;
    mac.update(encoded.as_bytes());
    mac.verify_slice(&signature)
        .map_err(|_| "invalid access token signature")?;
    let claims: AccessClaims = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| "malformed access token")?,
    )
    .map_err(|_| "malformed access token")?;
    if claims.expires_at <= Utc::now().timestamp() {
        return Err("access token expired");
    }
    Ok(claims)
}

/// Refresh tokens are opaque values. Only this digest is persisted, so a
/// database disclosure cannot be replayed as a session credential.
pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn new_refresh_token() -> String {
    format!("qvr1.{}", Uuid::now_v7().simple())
}

/// Raw, single-use "forgot password" token. Only its SHA-256 hash is ever
/// persisted (see `hash_password_reset_token`); the raw value is emailed
/// once to the account holder and never stored.
pub fn new_password_reset_token() -> String {
    format!("qvpr1.{}", Uuid::now_v7().simple())
}

pub fn hash_password_reset_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_token_round_trip_and_tamper_rejection() {
        let secret = b"test-secret-that-is-long-enough";
        let subject = Uuid::now_v7();
        let tenant = Uuid::now_v7();
        let token = issue_access_token(
            secret,
            subject,
            tenant,
            UserRole::Dispatcher,
            Duration::minutes(5),
        )
        .unwrap();
        let claims = verify_access_token(secret, &token).unwrap();
        assert_eq!(claims.subject, subject);
        assert_eq!(claims.tenant_id, tenant);
        assert_eq!(claims.role, UserRole::Dispatcher);
        assert!(verify_access_token(secret, &(token + "x")).is_err());
    }

    #[test]
    fn password_hashes_are_salted_and_verifiable() {
        let password = "correct horse battery staple";
        let hash = hash_password(password).unwrap();
        assert_ne!(hash, password);
        assert!(verify_password(&hash, password).is_ok());
        assert!(verify_password(&hash, "wrong password").is_err());
        assert!(hash_password("too-short").is_err());
    }

    #[test]
    fn refresh_token_is_opaque_and_hashes_deterministically() {
        let token = new_refresh_token();
        assert!(token.starts_with("qvr1."));
        assert_ne!(hash_refresh_token(&token), token);
        assert_eq!(hash_refresh_token(&token), hash_refresh_token(&token));
    }
}
