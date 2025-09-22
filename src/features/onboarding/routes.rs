use std::u64;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::features::{
    clients::EmailClient,
    onboarding::{
        get_client_ip, ip_to_bucket, parse_ip, sha256_hex,
        types::{AppState, PreparationReq, PreparationResp, WithEmailReq, WithEmailResp},
        OnboardingService, COOKIE_VISITOR,
    },
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
pub async fn preparation(
    req: HttpRequest,
    payload: web::Json<PreparationReq>,
    state: web::Data<AppState>,
    onboarding_service: web::Data<OnboardingService>,
) -> actix_web::Result<impl Responder> {
    // 1) visitor cookie
    let (visitor_id, maybe_cookie) =
        onboarding_service.read_or_set_visitor_cookie(req.cookie(COOKIE_VISITOR));

    print!("{:?}", payload.extra_data);

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
    responses(
        (status = 200, description = "An email with OTP and cookie is being sent to the user."),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Too many requests"),
    )
)]
#[post("/onboarding/with-email")]
pub async fn with_email(
    req: HttpRequest,
    payload: web::Json<WithEmailReq>,
    state: web::Data<AppState>,
    onboarding_service: web::Data<OnboardingService>,
    email_client: web::Data<EmailClient>,
) -> actix_web::Result<impl Responder> {
    // 1) verify the preparation cookie
    let _ = match onboarding_service.has_visitor_cookie(req.cookie(COOKIE_VISITOR)) {
        Some(id) => id,
        None => return Ok(HttpResponse::Forbidden().finish()),
    };

    let email = &payload.email;
    let limiter = state.limiter.lock().await;

    let k_email = format!("rl:email:{}", sha256_hex(email));
    let window_ms = 60_000u64;
    let limit_email = 3u32;

    // 3) check email sending rate limits
    if limiter
        .hit(&k_email, limit_email, window_ms)
        .await
        .unwrap_or(u64::MAX) // defaults to u64::MAX if I receive a RedisError
        >= limit_email as u64
    {
        return Ok(HttpResponse::TooManyRequests().finish());
    }

    let cookie = onboarding_service.send_otp(email, &email_client).await?;
    let mut resp = HttpResponse::Ok();
    resp.cookie(cookie);
    Ok(resp.json(WithEmailResp { ok: true }))
}

#[utoipa::path(
    post,
    path="/onboarding/otp-verification",
    tag="onboarding",
    responses(
        (status = 200, description = "Prepare user for authentication"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Too many requests"),
    )
)]
#[post("/onboarding/otp-verification")]
pub async fn otp_verification() -> actix_web::Result<impl Responder> {
    // let raw_cookie_value = "d0f6768480878a56aff1eee818ddb7852963753e08039d288a93b7bb4fc57121.zorIpwreOcaiXwVY3hCwMiH-HNdHiW2QaERb4vtPLF0";

    //     if let Some(nonce) = self.hmac_client.decode_cookie_value(raw_cookie_value) {
    //         println!("Decoded nonce: {}", nonce);
    //     } else {
    //         println!("Invalid signature!");
    //     }
    let resp = HttpResponse::Ok();
    Ok(resp)
}
