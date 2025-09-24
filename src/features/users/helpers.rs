use crate::utils::error::{Error, Result};
use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use num_traits::FromPrimitive;

pub const COOKIE_DEVICE_ID: &str = "__Host-device_id";
pub const COOKIE_USER_ID: &str = "__Host-user_id";

pub fn host_cookie(name: &str, value: String, max_age_seconds: i64, http_only: bool) -> Cookie<'_> {
    let mut c = Cookie::build(name.to_owned(), value)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .http_only(http_only)
        .finish();

    // Max-Age only if positive
    if max_age_seconds > 0 {
        c.set_max_age(CookieDuration::seconds(max_age_seconds));
    }
    c
}

/// Verify Argon2 hash that you stored on create().
pub fn verify_password(stored_hash: &str, plain: &str) -> Result<bool> {
    let parsed = PasswordHash::new(stored_hash)
        .map_err(|_| Error::Unexpected("invalid stored password hash".into()))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok())
}

// src/features/auth/ip_logging.rs
use crate::features::clients::MaxMindClient;
use sqlx::{
    types::{ipnetwork::IpNetwork, BigDecimal},
    PgPool,
};
use std::net::IpAddr;

pub async fn log_login_attempt(
    pool: &PgPool,
    maxmind: &MaxMindClient,
    user_id: Option<i64>,
    ip: Option<IpAddr>,
    success: bool,
) -> Result<()> {
    let mut country: Option<String> = None;
    let mut city: Option<String> = None;
    let mut asn_name: Option<String> = None;
    let mut latitude: Option<f64> = None;
    let mut longitude: Option<f64> = None;

    if let Some(ipaddr) = ip {
        if let Ok(info) = maxmind.lookup_all(ipaddr) {
            if let Some(c) = info.country {
                country = c
                    .country
                    .and_then(|c| c.names)
                    .and_then(|n| n.get("en").map(|s| s.to_string()));
            }
            if let Some(cityv) = info.city {
                city = cityv
                    .city
                    .and_then(|c| c.names)
                    .and_then(|n| n.get("en").map(|s| s.to_string()));
                if let Some(loc) = cityv.location {
                    latitude = loc.latitude;
                    longitude = loc.longitude;
                }
            }
            if let Some(asnv) = info.asn {
                asn_name = asnv.autonomous_system_organization.map(|s| s.to_string());
            }
        }
    }

    let ip_net: Option<IpNetwork> = ip.map(IpNetwork::from);
    let lat_bd: Option<BigDecimal> = latitude.and_then(BigDecimal::from_f64);
    let lon_bd: Option<BigDecimal> = longitude.and_then(BigDecimal::from_f64);

    sqlx::query!(
        r#"
        INSERT INTO login_attempts
          (user_id, success, ip_address, country, city, asn, latitude, longitude)
        VALUES
          ($1,     $2,      $3,         $4,      $5,   $6,  $7,       $8)
        "#,
        user_id,
        success,
        ip_net,
        country,
        city,
        asn_name,
        lat_bd,
        lon_bd
    )
    .execute(pool)
    .await
    .map_err(Error::from)?;

    Ok(())
}
