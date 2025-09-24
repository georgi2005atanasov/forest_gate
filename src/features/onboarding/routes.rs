use std::u64;

use actix_web::{
    cookie::{Cookie, SameSite},
    post, web, HttpRequest, HttpResponse, Responder,
};
use time::Duration;
use validator::Validate;

use crate::features::{
    clients::EmailClient,
    onboarding::{
        get_client_ip, ip_to_bucket, parse_ip, sha256_hex,
        types::{
            AppState, EmailVerificationReq, PreparationReq, PreparationResp, UserDetailsResp,
            WithEmailReq, WithEmailResp,
        },
        OnboardingService, EMAIL_PREFIX, INSTALL_PREFIX, IP_PREFIX, VISITOR_PREFIX,
    },
    users::types::UserDetailsReq,
};

/// region Cookies
pub const COOKIE_VISITOR: &str = "__Host-visitor_id";
pub const COOKIE_WITH_EMAIL: &str = "__Host-with_email";
pub const COOKIE_EMAIL_VERIFIED: &str = "__Host-email_verified";
pub const COOKIE_DEVICE_ID: &str = "__Host-device_id";
pub const COOKIE_USER_ID: &str = "__Host-user_id";
/// endregion Cookies

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
    let cookie_value = req.cookie(COOKIE_VISITOR).map(|c| c.value().to_string());
    let (visitor_id, maybe_value) =
        onboarding_service.read_or_set_visitor_cookie(cookie_value.as_deref());

    print!("{:?}", payload.extra_data);

    // 2) pick IP (header or peer) — if payload.ip is present, you can prefer server-detected IP instead
    let ip = get_client_ip(&req).or_else(|| payload.ip.as_ref().and_then(|s| parse_ip(s)));
    let ip_bucket = ip.as_ref().map(ip_to_bucket);

    // 3) install id from payload
    let install_id = &payload.extra_data.install_id;

    let k_visitor = format!("{}{}", VISITOR_PREFIX, sha256_hex(&visitor_id));
    let k_install = format!("{}{}", INSTALL_PREFIX, sha256_hex(install_id));
    let k_ip = ip_bucket
        .as_ref()
        .map(|b| format!("{}{}", IP_PREFIX, sha256_hex(b)));

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

    // Ensure device exists or create one
    let device = onboarding_service
        .ensure_device_from_preparation(&payload)
        .await?;
    let device_cookie = Cookie::build(COOKIE_DEVICE_ID, device.id.to_string())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(180))
        .path("/") // required for __Host- prefix (and do not set Domain)
        .finish();

    // 6) Build response
    let mut resp = HttpResponse::Ok();
    if let Some(value) = maybe_value {
        let cookie = Cookie::build(COOKIE_VISITOR, value)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::days(180))
            .path("/")
            .finish();
        resp.cookie(cookie);
    }
    resp.cookie(device_cookie);

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
    if let Err(errors) = payload.validate() {
        return Ok(HttpResponse::BadRequest().json(errors));
    }

    // 1) verify the preparation cookie
    let _ = match onboarding_service.has_valid_cookie(req.cookie(COOKIE_VISITOR)) {
        Some(id) => id,
        None => return Ok(HttpResponse::Forbidden().finish()),
    };

    let limiter = state.limiter.lock().await;

    let email = &payload.email;
    let k_email = format!("{}{}", EMAIL_PREFIX, sha256_hex(email));
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

    let cookie_value = onboarding_service.send_otp(email, &email_client).await?;
    let cookie = Cookie::build(COOKIE_WITH_EMAIL, cookie_value)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(10)) // same as OTP TTL
        .path("/") // required for __Host-*
        .finish();

    let mut resp = HttpResponse::Ok();
    resp.cookie(cookie);
    Ok(resp.json(WithEmailResp { ok: true }))
}

#[utoipa::path(
    post,
    path="/onboarding/otp-verification",
    tag="onboarding",
    responses(
        (status = 200, description = "Email verified successfuly"),
        (status = 403, description = "Forbidden"),
    )
)]
#[post("/onboarding/otp-verification")]
pub async fn otp_verification(
    req: HttpRequest,
    payload: web::Json<EmailVerificationReq>,
    onboarding_service: web::Data<OnboardingService>,
) -> actix_web::Result<impl Responder> {
    if let Err(errors) = payload.validate() {
        return Ok(HttpResponse::BadRequest().json(errors));
    }

    let cookie_value = req.cookie(COOKIE_WITH_EMAIL).map(|c| c.value().to_string());

    let cookie_value = onboarding_service
        .verify_email(&payload.email, &payload.code, cookie_value.as_deref())
        .await?;

    let cookie = Cookie::build(COOKIE_EMAIL_VERIFIED, cookie_value)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(180))
        .path("/") // required for __Host-prefix (and do not set Domain)
        .finish();

    let mut resp = HttpResponse::Ok();
    resp.cookie(cookie);
    Ok(resp.finish())
}

#[utoipa::path(
    post,
    path="/onboarding/user-details",
    tag="onboarding",
    responses(
        (status = 200, description = "User registered"),
        (status = 403, description = "Forbidden"),
    )
)]
#[post("/onboarding/user-details")]
pub async fn user_details(
    req: HttpRequest,
    payload: web::Json<UserDetailsReq>,
    onboarding_service: web::Data<OnboardingService>,
) -> actix_web::Result<impl Responder> {
    if let Err(errors) = payload.validate() {
        return Ok(HttpResponse::BadRequest().json(errors));
    }

    // 1) device id + email_verified cookies check (decoded signature = actual email)
    let device_id: i64 = match req.cookie(COOKIE_DEVICE_ID) {
        Some(c) => match c.value().parse::<i64>() {
            Ok(id) => id,
            Err(_) => return Ok(HttpResponse::Forbidden().finish()),
        },
        None => return Ok(HttpResponse::Forbidden().finish()),
    };

    if let Some(cookie) = req.cookie(COOKIE_EMAIL_VERIFIED) {
        // if cookie {
        println!("cookie email: {:?}", cookie.value());
        // }
    } else {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let email = match onboarding_service.has_valid_cookie(req.cookie(COOKIE_EMAIL_VERIFIED)) {
        Some(id) => id,
        None => return Ok(HttpResponse::Forbidden().finish()),
    };

    println!("device id: {}\n email: {}", device_id, email);

    // 2) create user + user_devices
    let (user_id, _device_id) = onboarding_service
        .ensure_user_with_device(
            &device_id,
            &email,
            &payload.password,
            &payload.confirm_password,
        )
        .await?;

    let user_cookie = Cookie::build(COOKIE_USER_ID, user_id.to_string())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(180))
        .path("/") // required for __Host- prefix (and do not set Domain)
        .finish();
    // 4) generate jwt and store it in cookie and automatically login the user.

    let mut resp = HttpResponse::Ok();
    resp.cookie(user_cookie);
    Ok(resp.json(UserDetailsResp { user_id: user_id }))
}
