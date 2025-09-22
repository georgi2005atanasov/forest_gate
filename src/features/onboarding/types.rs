use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;

use crate::features::onboarding::repo::RateLimiter;

pub struct AppState {
    pub limiter: Mutex<RateLimiter>,
}

// ====== Request structs (camelCase JSON) ======
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    brand: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentData {
    brands: Option<Vec<Brand>>,
    mobile: Option<bool>,
    platform: Option<String>,
    architecture: Option<String>,
    bitness: Option<String>,
    model: Option<String>,
    platform_version: Option<String>,
    ua_full_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Screen {
    width: u32,
    height: u32,
    color_depth: u8,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    device_memory_gb: Option<u32>,
    hardware_concurrency: Option<u32>,
    max_touch_points: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Webgl {
    vendor: Option<String>,
    renderer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
pub(super) struct PreparationReq {
    pub app_version: String,
    pub fingerprint: String,
    pub extra_data: StableFingerprintData,
    // if you also want to accept ip in body (optional)
    pub ip: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreparationResp {
    pub ok: bool,
    pub visitor_id: String,
}
