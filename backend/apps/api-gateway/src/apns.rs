// =============================================================================
// File:           backend/apps/api-gateway/src/apns.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-16
// Version:        0.1.0
//
// Description:
//   Real Apple Push Notification service (APNs) HTTP/2 client. Builds a
//   provider authentication token (ES256 JWT, per Apple's token-based
//   authentication scheme — https://developer.apple.com/documentation/
//   usernotifications/establishing-a-token-based-connection-to-apns) and
//   POSTs a JSON alert payload to `/3/device/{token}` on Apple's sandbox or
//   production push gateway, selecting the `apns-topic` (bundle id) that
//   matches the app variant (Courier vs Customer) the device token was
//   issued by.
//
//   Configuration is entirely environment-driven (see `from_env`) so this
//   client is inert — `AppState.apns` stays `None` — until real Apple
//   Developer Program credentials are supplied; nothing here fabricates or
//   assumes a credential. Android/FCM delivery is a separate, still-unwired
//   concern (see BACKEND_BACKLOG.md).
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use qervon_domain::AppVariant;
use serde::Serialize;

/// Apple recommends generating a provider token at most once every 20
/// minutes and treats a token as valid for up to 60 minutes; refreshing at
/// 45 minutes keeps a wide safety margin on both ends.
const TOKEN_LIFETIME: Duration = Duration::from_secs(45 * 60);

#[derive(Debug)]
pub enum ApnsError {
    /// The device token's app variant has no configured bundle id.
    MissingBundleId(AppVariant),
    Jwt(jsonwebtoken::errors::Error),
    Http(reqwest::Error),
    /// Apple accepted the connection but rejected the push (e.g.
    /// `BadDeviceToken`, `Unregistered`, `TopicDisallowed`). Carries the
    /// HTTP status and Apple's JSON `reason` body so callers can log
    /// something actionable instead of a bare "failed".
    Rejected {
        status: reqwest::StatusCode,
        reason: String,
    },
}

impl std::fmt::Display for ApnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBundleId(variant) => {
                write!(f, "no APNS bundle id configured for app variant {variant}")
            }
            Self::Jwt(error) => write!(f, "failed to sign APNs provider token: {error}"),
            Self::Http(error) => write!(f, "APNs request failed: {error}"),
            Self::Rejected { status, reason } => {
                write!(f, "APNs rejected the push ({status}): {reason}")
            }
        }
    }
}

impl std::error::Error for ApnsError {}

/// Configuration loaded once at startup from `APNS_*` environment
/// variables. See `.env.example` for the full list and where each value
/// comes from in the Apple Developer / App Store Connect portals.
pub struct ApnsConfig {
    pub team_id: String,
    pub key_id: String,
    /// PEM-encoded PKCS#8 EC private key — the exact contents of the `.p8`
    /// file downloaded once, at creation time, from Certificates,
    /// Identifiers & Profiles → Keys in the Apple Developer portal (Apple
    /// does not let you re-download it afterwards).
    pub private_key_pem: String,
    pub bundle_id_courier: Option<String>,
    pub bundle_id_customer: Option<String>,
    /// Sandbox is Apple's development/TestFlight-adjacent gateway; a device
    /// token issued to a build without the production entitlement will be
    /// rejected by the production gateway and vice versa.
    pub use_sandbox: bool,
}

impl ApnsConfig {
    fn bundle_id_for(&self, variant: AppVariant) -> Result<&str, ApnsError> {
        let configured = match variant {
            AppVariant::Courier => self.bundle_id_courier.as_deref(),
            AppVariant::Customer => self.bundle_id_customer.as_deref(),
        };
        configured.ok_or(ApnsError::MissingBundleId(variant))
    }
}

struct CachedToken {
    jwt: String,
    issued_at: Instant,
}

pub struct ApnsClient {
    config: ApnsConfig,
    http: reqwest::Client,
    cached_token: RwLock<Option<CachedToken>>,
}

#[derive(Serialize, serde::Deserialize)]
struct ApnsClaims {
    iss: String,
    iat: i64,
}

#[derive(Serialize)]
struct ApnsAlert<'a> {
    title: &'a str,
    body: &'a str,
}

#[derive(Serialize)]
struct ApnsAps<'a> {
    alert: ApnsAlert<'a>,
    sound: &'a str,
}

#[derive(Serialize)]
struct ApnsPayload<'a> {
    aps: ApnsAps<'a>,
}

impl ApnsClient {
    /// Builds a client from `APNS_TEAM_ID`, `APNS_KEY_ID`,
    /// `APNS_PRIVATE_KEY_PATH` (or `APNS_PRIVATE_KEY_PEM` inline), and at
    /// least one of `APNS_BUNDLE_ID_COURIER` / `APNS_BUNDLE_ID_CUSTOMER`.
    /// Returns `None` (not an error) when any required piece is missing —
    /// APNs stays an opt-in feature, matching every other outbound provider
    /// in this codebase (SMS/payment/generic push).
    pub fn from_env() -> Option<Self> {
        let team_id = env_non_empty("APNS_TEAM_ID")?;
        let key_id = env_non_empty("APNS_KEY_ID")?;
        let private_key_pem = std::env::var("APNS_PRIVATE_KEY_PATH")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .and_then(|path| std::fs::read_to_string(path).ok())
            .or_else(|| env_non_empty("APNS_PRIVATE_KEY_PEM"))?;
        let bundle_id_courier = env_non_empty("APNS_BUNDLE_ID_COURIER");
        let bundle_id_customer = env_non_empty("APNS_BUNDLE_ID_CUSTOMER");
        if bundle_id_courier.is_none() && bundle_id_customer.is_none() {
            tracing::warn!(
                "APNS_TEAM_ID/APNS_KEY_ID/APNS_PRIVATE_KEY are set but neither \
                 APNS_BUNDLE_ID_COURIER nor APNS_BUNDLE_ID_CUSTOMER is; APNs stays disabled"
            );
            return None;
        }
        let use_sandbox = std::env::var("APNS_ENVIRONMENT")
            .map(|value| value.eq_ignore_ascii_case("sandbox"))
            .unwrap_or(true);

        Some(Self {
            config: ApnsConfig {
                team_id,
                key_id,
                private_key_pem,
                bundle_id_courier,
                bundle_id_customer,
                use_sandbox,
            },
            http: reqwest::Client::new(),
            cached_token: RwLock::new(None),
        })
    }

    fn provider_token(&self) -> Result<String, ApnsError> {
        if let Ok(cached) = self.cached_token.read() {
            if let Some(cached) = cached.as_ref() {
                if cached.issued_at.elapsed() < TOKEN_LIFETIME {
                    return Ok(cached.jwt.clone());
                }
            }
        }

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.config.key_id.clone());
        let claims = ApnsClaims {
            iss: self.config.team_id.clone(),
            iat: chrono::Utc::now().timestamp(),
        };
        let encoding_key = EncodingKey::from_ec_pem(self.config.private_key_pem.as_bytes())
            .map_err(ApnsError::Jwt)?;
        let jwt = encode(&header, &claims, &encoding_key).map_err(ApnsError::Jwt)?;

        if let Ok(mut cached) = self.cached_token.write() {
            *cached = Some(CachedToken {
                jwt: jwt.clone(),
                issued_at: Instant::now(),
            });
        }
        Ok(jwt)
    }

    fn host(&self) -> &'static str {
        if self.config.use_sandbox {
            "api.sandbox.push.apple.com"
        } else {
            "api.push.apple.com"
        }
    }

    /// Sends a single alert push to one device token. Fire this per token,
    /// not per user — a user can have several registered devices.
    pub async fn send(
        &self,
        device_token: &str,
        app_variant: AppVariant,
        title: &str,
        body: &str,
    ) -> Result<(), ApnsError> {
        let bundle_id = self.config.bundle_id_for(app_variant)?;
        let jwt = self.provider_token()?;
        let payload = ApnsPayload {
            aps: ApnsAps {
                alert: ApnsAlert { title, body },
                sound: "default",
            },
        };

        let response = self
            .http
            .post(format!("https://{}/3/device/{device_token}", self.host()))
            .bearer_auth(jwt)
            .header("apns-topic", bundle_id)
            .header("apns-push-type", "alert")
            .header("apns-priority", "10")
            .json(&payload)
            .send()
            .await
            .map_err(ApnsError::Http)?;

        if response.status().is_success() {
            return Ok(());
        }
        let status = response.status();
        let reason = response
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|body| {
                body.get("reason")
                    .and_then(|r| r.as_str().map(String::from))
            })
            .unwrap_or_else(|| "unknown".to_string());
        Err(ApnsError::Rejected { status, reason })
    }
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keypair_pem() -> (String, String) {
        use openssl::{
            ec::{EcGroup, EcKey},
            nid::Nid,
            pkey::PKey,
        };

        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).expect("P-256 curve");
        let key = EcKey::generate(&group).expect("generate ephemeral test key");
        let key = PKey::from_ec_key(key).expect("convert ephemeral test key");
        let private_key =
            String::from_utf8(key.private_key_to_pem_pkcs8().expect("encode private key"))
                .expect("private key is PEM text");
        let public_key = String::from_utf8(key.public_key_to_pem().expect("encode public key"))
            .expect("public key is PEM text");
        (private_key, public_key)
    }

    fn test_client(private_key_pem: String) -> ApnsClient {
        ApnsClient {
            config: ApnsConfig {
                team_id: "TEST1234TM".into(),
                key_id: "TESTKEY123".into(),
                private_key_pem,
                bundle_id_courier: Some("com.qervon.ios.courier".into()),
                bundle_id_customer: None,
                use_sandbox: true,
            },
            http: reqwest::Client::new(),
            cached_token: RwLock::new(None),
        }
    }

    #[test]
    fn signs_a_provider_token_with_expected_header_and_claims() {
        let (private_key, public_key) = test_keypair_pem();
        let client = test_client(private_key);
        let jwt = client.provider_token().expect("sign provider token");

        let mut validation = jsonwebtoken::Validation::new(Algorithm::ES256);
        validation.required_spec_claims.clear();
        let decoding_key =
            jsonwebtoken::DecodingKey::from_ec_pem(public_key.as_bytes()).expect("decoding key");
        // Deliberately does NOT disable signature validation: this proves
        // the JWT is actually signed correctly with the matching private
        // key, not just shaped correctly.
        let decoded = jsonwebtoken::decode::<ApnsClaims>(&jwt, &decoding_key, &validation)
            .expect("decode and verify signed token");

        assert_eq!(decoded.header.alg, Algorithm::ES256);
        assert_eq!(decoded.header.kid, Some("TESTKEY123".to_string()));
        assert_eq!(decoded.claims.iss, "TEST1234TM");
    }

    #[test]
    fn caches_the_provider_token_across_calls() {
        let (private_key, _) = test_keypair_pem();
        let client = test_client(private_key);
        let first = client.provider_token().expect("first token");
        let second = client.provider_token().expect("second token");
        assert_eq!(first, second, "token should be reused within its lifetime");
    }

    #[test]
    fn reports_missing_bundle_id_for_unconfigured_app_variant() {
        let (private_key, _) = test_keypair_pem();
        let client = test_client(private_key);
        let error = client
            .config
            .bundle_id_for(AppVariant::Customer)
            .unwrap_err();
        assert!(matches!(
            error,
            ApnsError::MissingBundleId(AppVariant::Customer)
        ));
    }

    /// Opt-in smoke test against Apple's *real* sandbox gateway, using real
    /// `APNS_*` credentials from the environment (never hardcoded — see
    /// `.env.example`). Sends to a syntactically valid but definitely-
    /// unregistered 64-hex-char device token. A real credential problem
    /// (bad team id, bad key id, key/team mismatch, wrong topic) surfaces as
    /// `InvalidProviderToken` or `TopicDisallowed`; a *working* credential
    /// against a fake token surfaces as `BadDeviceToken` — proving the JWT
    /// signature and topic were accepted by Apple before the (expected)
    /// device lookup failure. Never prints the key or the signed JWT.
    #[tokio::test]
    #[ignore = "requires real APNS_* env vars; run with --ignored after setting them"]
    async fn real_credentials_are_accepted_by_apples_sandbox_gateway() {
        let client = ApnsClient::from_env().expect(
            "APNS_TEAM_ID/APNS_KEY_ID/APNS_PRIVATE_KEY_PATH(or _PEM)/APNS_BUNDLE_ID_* must be set",
        );
        let placeholder_token = "0".repeat(64);
        let result = client
            .send(
                &placeholder_token,
                AppVariant::Courier,
                "Qervon Smoke Test",
                "İlk gerçek APNs kimlik doğrulama testi",
            )
            .await;
        match result {
            Err(ApnsError::Rejected { status, reason }) => {
                println!("Apple responded: HTTP {status} — reason: {reason}");
                assert_eq!(
                    reason, "BadDeviceToken",
                    "expected only the placeholder device token to be rejected — a different \
                     reason (e.g. InvalidProviderToken, TopicDisallowed) means the credentials \
                     themselves are the problem, not the fake token"
                );
            }
            other => panic!(
                "expected Apple to reject the placeholder device token with BadDeviceToken, got: {other:?}"
            ),
        }
    }
}
