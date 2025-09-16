use actix_web::{web, HttpResponse, Result, get};
use serde_json::json;

#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}

#[get("/version")]
pub async fn version() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME")
    })))
}