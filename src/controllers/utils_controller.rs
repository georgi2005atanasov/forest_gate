use actix_web::{HttpResponse, Result, get};
use serde_json::json;
use utoipa;

#[utoipa::path(
    get,
    path = "/health",
    tag = "UtilsController",
    responses(
        (status = 200, description = "Get API health + other metadata")
    )
)]
#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}

#[utoipa::path(
    get,
    path = "/version",
    tag = "UtilsController",
    responses(
        (status = 200, description = "Get API version + other metadata")
    )
)]
#[get("/version")]
pub async fn version() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME")
    })))
}