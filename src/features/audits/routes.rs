use actix_web::{cookie::{Cookie, SameSite}, post, web, HttpResponse, Result};
use serde_json::json;
use time::Duration;
use uuid::Uuid;

use crate::{
    features::audits::{types::AuditEvent, AuditService},
    utils::error::Error,
};

/// region Cookies
pub const COOKIE_TRACKING: &str = "auth-track_interaction";

#[utoipa::path(
    post,
    path = "/audit/init",
    tag = "audit",
    responses(
        (status = 200, description = "Initialize cookie for tracking user interaction. 
        You may want to verify user_id, device_id and session_id, make a row in the user_interactions table,
        but for now we will go without this validation in order to be more flexible for future cases."),
        (status = 400, description = "Bad request")
    )
)]
#[post("/audit/init")]
pub async fn audit_init() -> Result<HttpResponse> {
    let cookie =Cookie::build(COOKIE_TRACKING, Uuid::new_v4().to_string())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(15))
        .path("/")
        .finish();

    let mut resp = HttpResponse::Ok();
    resp.cookie(cookie);
    Ok(resp.json(json!({
        "success": true
    })))
}

#[utoipa::path(
    post,
    path = "/audit/batch",
    tag = "audit",
    responses(
        (status = 200, description = "Audit user session"),
        (status = 400, description = "Bad Request")
    )
)]
#[post("/audit/batch")]
pub async fn audit_batch(
    audit_service: web::Data<AuditService>,
    body: Result<web::Json<AuditEvent>, actix_web::Error>,
) -> Result<HttpResponse> {
    let body: std::result::Result<web::Json<AuditEvent>, actix_web::Error> = body.map_err(|e| {
        tracing::error!("Invalid request body: {:?}", e);
        e
    });

    let body = body.map_err(|e| Error::Validation(format!("bad body: {e}")))?;

    if body.event.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    audit_service
        .append_events(&body.interaction_id, &body.event)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
