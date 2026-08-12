// =============================================================================
// File:           backend/apps/api-gateway/src/rate_limit.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Rate-limiting key extraction for the Qervon API gateway. Provides an
//   infallible client-IP extractor so that requests never receive an
//   internal-error response when no proxy/connect-info is available (local
//   development, tests, or misconfigured reverse proxies).
//
// Specification:
//   QAS-000004, QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::ConnectInfo;
use axum::http::Request;
use tower_governor::{key_extractor::KeyExtractor, GovernorError};

/// Extracts the caller's IP address for rate-limiting purposes.
///
/// Resolution order: `X-Forwarded-For`, then `X-Real-IP` (both only trusted
/// when the API sits behind a reverse proxy that sets them), then the
/// transport-level `ConnectInfo`/`SocketAddr` extension, and finally a fixed
/// "unknown origin" bucket. Unlike `tower_governor`'s built-in extractors,
/// this implementation never fails: an unresolved origin degrades to a
/// shared low-priority bucket instead of turning every request into a 500.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClientIpKeyExtractor;

impl KeyExtractor for ClientIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        Ok(client_ip(req))
    }
}

fn client_ip<T>(req: &Request<T>) -> IpAddr {
    if let Some(ip) = header_ip(req, "x-forwarded-for", true) {
        return ip;
    }
    if let Some(ip) = header_ip(req, "x-real-ip", false) {
        return ip;
    }
    if let Some(connect_info) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect_info.0.ip();
    }
    if let Some(addr) = req.extensions().get::<SocketAddr>() {
        return addr.ip();
    }
    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
}

fn header_ip<T>(req: &Request<T>, name: &str, is_forwarded_list: bool) -> Option<IpAddr> {
    let value = req.headers().get(name)?.to_str().ok()?;
    if is_forwarded_list {
        value.split(',').find_map(|part| part.trim().parse().ok())
    } else {
        value.trim().parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request as HttpRequest;

    fn request_with_header(name: &str, value: &str) -> HttpRequest<()> {
        HttpRequest::builder()
            .header(name, value)
            .body(())
            .expect("valid request")
    }

    #[test]
    fn prefers_x_forwarded_for_over_x_real_ip() {
        let mut request = request_with_header("x-forwarded-for", "203.0.113.10, 10.0.0.1");
        request
            .headers_mut()
            .insert("x-real-ip", "198.51.100.5".parse().unwrap());
        assert_eq!(
            client_ip(&request),
            "203.0.113.10".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn falls_back_to_x_real_ip() {
        let request = request_with_header("x-real-ip", "198.51.100.5");
        assert_eq!(
            client_ip(&request),
            "198.51.100.5".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn falls_back_to_connect_info() {
        let mut request = HttpRequest::builder().body(()).expect("valid request");
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 4000))));
        assert_eq!(client_ip(&request), "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn falls_back_to_unspecified_when_no_signal_present() {
        let request = HttpRequest::builder().body(()).expect("valid request");
        assert_eq!(client_ip(&request), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn extract_never_fails() {
        let request = HttpRequest::builder().body(()).expect("valid request");
        assert!(ClientIpKeyExtractor.extract(&request).is_ok());
    }
}
