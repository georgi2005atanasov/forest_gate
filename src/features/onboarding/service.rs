use std::net::IpAddr;

use actix_web::{
    cookie::{Cookie, SameSite},
    HttpRequest,
};
use time::Duration;
use uuid::Uuid;

use crate::utils::crypto::ClientHMAC;

const COOKIE_NAME: &str = "__Host-visitor_id";

pub(super) fn read_or_set_visitor_cookie(
    req: &HttpRequest,
    hmac_client: &ClientHMAC,
) -> (String, Option<Cookie<'static>>) {
    if let Some(c) = req.cookie(COOKIE_NAME) {
        if let Some(id) = hmac_client.decode_cookie_value(c.value()) {
            return (id, None);
        }
        // invalid or tampered; fall through and re-issue
    }

    let new_id = Uuid::new_v4().to_string();
    let value = hmac_client.encode_cookie_value(&new_id);

    let cookie = Cookie::build(COOKIE_NAME, value)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(180))
        .path("/") // required for __Host- prefix (and do not set Domain)
        .finish();

    (new_id, Some(cookie))
}

// ====== IP helpers ======
pub(super) fn parse_ip(s: &str) -> Option<IpAddr> {
    s.trim().parse::<IpAddr>().ok()
}
pub(super) fn get_client_ip(req: &HttpRequest) -> Option<IpAddr> {
    // Prefer Forwarded: for=...
    if let Some(h) = req.headers().get("forwarded") {
        if let Ok(v) = h.to_str() {
            for part in v.split(';').flat_map(|x| x.split(',')) {
                let p = part.trim();
                if let Some(val) = p.strip_prefix("for=") {
                    let val = val.trim_matches('"');
                    if let Some(ip) = parse_ip(val) {
                        return Some(ip);
                    }
                }
            }
        }
    }
    if let Some(h) = req.headers().get("x-forwarded-for") {
        if let Ok(v) = h.to_str() {
            if let Some(first) = v.split(',').next() {
                if let Some(ip) = parse_ip(first) {
                    return Some(ip);
                }
            }
        }
    }
    req.peer_addr().map(|p| p.ip())
}

/// Reduce IP to a coarse bucket (/24 for v4, /64 for v6)
pub(super) fn ip_to_bucket(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            format!("{}.{}.{}.0/24", o[0], o[1], o[2])
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

pub(super) fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    hex::encode(h.finalize())
}
