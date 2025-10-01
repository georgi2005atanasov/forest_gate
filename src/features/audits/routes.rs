use actix_web::{post, web, HttpResponse, Result};

use crate::{
    features::audits::{types::AuditEvent, AuditService},
    utils::error::Error,
};

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
