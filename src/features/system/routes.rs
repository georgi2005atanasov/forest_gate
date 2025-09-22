use crate::features::clients::EmailClient;
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
    path = "/system/health",
    tag = "SystemController",
    responses(
        (status = 200, description = "Get API health + other metadata")
    )
)]
#[get("/system/health")]
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
    path = "/system/version",
    tag = "SystemController",
    responses(
        (status = 200, description = "Get API version + other metadata")
    )
)]
#[get("/system/version")]
pub async fn version() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME")
    })))
}

#[utoipa::path(
    get,
    path = "/system/config",
    tag = "SystemController",
    responses(
        (status = 200, description = "Get system config", body = ConfigDto)
    )
)]
#[get("/system/config")]
pub async fn config(service: web::Data<ConfigService>) -> Result<HttpResponse> {
    let cfg = service.get().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(ConfigDto::from(cfg)))
}

#[utoipa::path(
    put,
    path = "/system/config",
    tag = "SystemController",
    request_body = ConfigDto,
    responses(
        (status = 200, description = "Update system config successfully"),
        (status = 400, description = "Invalid config input")
    )
)]
#[put("/system/config")]
pub async fn update_config(
    service: web::Data<ConfigService>,
    email_client: web::Data<EmailClient>,
    payload: web::Json<ConfigDto>,
) -> Result<HttpResponse> {
    let dto = payload.into_inner();

    service
        .update(&dto)
        .await
        .map_err(ErrorInternalServerError)?;

    if let Ok(recipient) = std::env::var("NOTIFY_EMAIL") {
        let subject = "Config updated";
        let text = format!(
            "Config has been updated.\n\
             allow_recovery_codes: {}\n\
             allow_refresh_tokens: {}\n\
             token_validity_seconds: {}\n\
             refresh_token_validity_seconds: {}\n\
             ai_model: {}\n\
             vector_similarity_threshold: {}",
            dto.allow_recovery_codes,
            dto.allow_refresh_tokens,
            dto.token_validity_seconds,
            dto.refresh_token_validity_seconds,
            dto.ai_model,
            dto.vector_similarity_threshold
        );

        if let Err(e) = email_client
            .send_text_and_html(&recipient, subject, Some(&text), None)
            .await
        {
            tracing::error!("Failed to send config update email: {e}");
        }
    }

    Ok(HttpResponse::Ok().json(json!({"status": "updated"})))
}
