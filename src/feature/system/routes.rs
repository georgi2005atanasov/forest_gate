use actix_web::error::ErrorInternalServerError;
use actix_web::{get, put, web, HttpResponse, Result};
use chrono::Utc;
use serde_json::json;
use sysinfo::System;
use utoipa;

use super::ConfigDto;
use super::ConfigService;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "SystemController",
    responses(
        (status = 200, description = "Get API health + other metadata")
    )
)]
#[get("/health")]
pub async fn health() -> Result<HttpResponse> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let system_name = System::name();
    let kernel_version = System::kernel_version();
    let os_version = System::os_version();
    let host_name = System::host_name();
    let available_memory = sys.available_memory();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let cpu_usage = sys
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect::<Vec<f32>>();

    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "host": {
            "system_name": system_name,
            "kernel_version": kernel_version,
            "os_version": os_version,
            "host_name": host_name,
            "memory": {
                "total_kb": total_memory,
                "available_kb": available_memory,
                "used_kb": used_memory
            },
            "cpu_usage_per_core": cpu_usage
        }
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/version",
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
    path = "/api/v1/config",
    tag = "SystemController",
    responses(
        (status = 200, description = "Get system config", body = ConfigDto)
    )
)]
#[get("/config")]
pub async fn config(service: web::Data<ConfigService>) -> Result<HttpResponse> {
    let cfg = service.get().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(ConfigDto::from(cfg)))
}

#[utoipa::path(
    put,
    path = "/api/v1/config",
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
    let entity = payload.into_inner();
    service
        .update(&entity)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(json!({"status": "updated"})))
}
