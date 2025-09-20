use actix_web::error::ErrorInternalServerError;
use actix_web::{get, put, web, HttpResponse, Result};
use serde::Deserialize;
use serde_json::json;
use utoipa;

use super::ConfigDto;
use super::ConfigService;

#[utoipa::path(
    get,
    path = "/health",
    tag = "SystemController",
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
    tag = "SystemController",
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

#[utoipa::path(
    get,
    path = "/config",
    tag = "SystemController",
    responses(
        (status = 200, description = "Get system config", body = ConfigDto)
    )
)]
#[get("/config")]
pub async fn get_config(service: web::Data<ConfigService>) -> Result<HttpResponse> {
    let cfg = service.get().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(ConfigDto::from(cfg)))
}

#[utoipa::path(
    put,
    path = "/config",
    tag = "SystemController",
    request_body = ConfigDto,
    responses(
        (status = 200, description = "Update system config successfully"),
        (status = 400, description = "Invalid config input")
    )
)]
#[put("/config")]
pub async fn update_config(
    service: web::Data<ConfigService>,
    payload: web::Json<ConfigDto>,
) -> Result<HttpResponse> {
    let entity = payload.into_inner().into();
    service.update(&entity).await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(json!({"status": "updated"})))
}