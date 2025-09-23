use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;
use validator::Validate;

use crate::features::onboarding::RateLimiter;

pub struct AppState {
    pub limiter: Mutex<RateLimiter>,
}

// ====== Request structs (camelCase JSON) ======
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    pub brand: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentData {
    pub brands: Option<Vec<Brand>>,
    pub mobile: Option<bool>,
    pub platform: Option<String>,
    pub architecture: Option<String>,
    pub bitness: Option<String>,
    pub model: Option<String>,
    pub platform_version: Option<String>,
    pub ua_full_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub color_depth: u8,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub device_memory_gb: Option<u32>,
    pub hardware_concurrency: Option<u32>,
    pub max_touch_points: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Webgl {
    pub vendor: Option<String>,
    pub renderer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StableFingerprintData {
    pub user_agent: String,
    pub user_agent_data: Option<UserAgentData>,
    pub primary_language: Option<String>,
    pub languages: Vec<String>,
    pub time_zone: Option<String>,
    pub time_zone_offset_minutes: i32,
    pub screen: Screen,
    pub hardware: Hardware,
    pub webgl: Option<Webgl>,
    pub canvas_hash: Option<String>,
    pub install_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PreparationReq {
    pub app_version: String,
    pub fingerprint: String,
    pub extra_data: StableFingerprintData,
    // if you also want to accept ip in body (optional)
    pub ip: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreparationResp {
    pub(super) ok: bool,
    pub(super) visitor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub(super) struct WithEmailReq {
    #[validate(email)]
    pub(super) email: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WithEmailResp {
    pub(super) ok: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub(super) struct EmailVerificationReq {
    // can be further more checked, but for speed purposes we leave it :)
    #[validate(email)]
    pub(super) email: String,
    #[validate(length(min = 6, max = 6))]
    pub(super) code: String,
}
