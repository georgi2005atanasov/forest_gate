use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use super::{get_client_ip, ip_to_bucket, parse_ip, read_or_set_visitor_cookie, sha256_hex};
use crate::{
    features::onboarding::types::{AppState, PreparationReq, PreparationResp},
    utils::crypto::ClientHMAC,
};

#[utoipa::path(
    get,
    path = "/onboarding/preparation",
    tag = "onboarding",
    responses(
        (status = 200, description = "Prepare user for authentication"),
        (status = 429, description = "Too many requests"),
    )
)]
#[post("/onboarding/preparation")]
async fn preparation(
    req: HttpRequest,
    payload: web::Json<PreparationReq>,
    state: web::Data<AppState>,
    hmac_client: web::Data<ClientHMAC>,
) -> actix_web::Result<impl Responder> {
    // 1) visitor cookie
    let (visitor_id, maybe_cookie) = read_or_set_visitor_cookie(&req, &hmac_client);

    // 2) pick IP (header or peer) — if payload.ip is present, you can prefer server-detected IP instead
    let ip = get_client_ip(&req).or_else(|| payload.ip.as_ref().and_then(|s| parse_ip(s)));
    let ip_bucket = ip.as_ref().map(ip_to_bucket);

    // 3) install id from payload
    let install_id = &payload.extra_data.install_id;

    let k_visitor = format!("rl:prep:v1:visitor:{}", sha256_hex(&visitor_id));
    let k_install = format!("rl:prep:v1:install:{}", sha256_hex(install_id));
    let k_ip = ip_bucket
        .as_ref()
        .map(|b| format!("rl:prep:v1:ip:{}", sha256_hex(b)));

    // 5) Apply rate limits (simple sequential calls)
    // tune as you like
    let window_ms = 60_000u64;
    let limit_cookie_install = 10u32; // per 60s
    let limit_ip = 100u32; // per 60s

    let limiter = state.limiter.lock().await;

    // cookie
    if limiter
        .hit(&k_visitor, limit_cookie_install, window_ms)
        .await
        .unwrap_or(u64::MAX)
        >= limit_cookie_install as u64
    {
        return Ok(HttpResponse::TooManyRequests().finish());
    }

    // install
    if limiter
        .hit(&k_install, limit_cookie_install, window_ms)
        .await
        .unwrap_or(u64::MAX)
        >= limit_cookie_install as u64
    {
        return Ok(HttpResponse::TooManyRequests().finish());
    }

    // ip (if present)
    if let Some(k) = &k_ip {
        if limiter
            .hit(k, limit_ip, window_ms)
            .await
            .unwrap_or(u64::MAX)
            >= limit_ip as u64
        {
            return Ok(HttpResponse::TooManyRequests().finish());
        }
    }

    // 6) Build response
    let mut resp = HttpResponse::Ok();
    if let Some(c) = maybe_cookie {
        resp.cookie(c);
    }

    Ok(resp.json(PreparationResp {
        ok: true,
        visitor_id,
    }))
}

#[utoipa::path(
    post,
    path="/onboarding/with-email",
    tag="onboarding",
    responses=(
        (status = 200, description = "Prepare user for authentication"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Too many requests"),
    )
)]
#[post("/onboarding/with-email")]
pub async fn with_email() {}
