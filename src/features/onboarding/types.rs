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
pub(super) struct Brand {
    pub(super) brand: String,
    pub(super) version: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserAgentData {
    pub(super) brands: Option<Vec<Brand>>,
    pub(super) mobile: Option<bool>,
    pub(super) platform: Option<String>,
    pub(super) architecture: Option<String>,
    pub(super) bitness: Option<String>,
    pub(super) model: Option<String>,
    pub(super) platform_version: Option<String>,
    pub(super) ua_full_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct Screen {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) color_depth: u8,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct Hardware {
    pub(super) device_memory_gb: Option<u32>,
    pub(super) hardware_concurrency: Option<u32>,
    pub(super) max_touch_points: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct Webgl {
    pub(super) vendor: Option<String>,
    pub(super) renderer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct StableFingerprintData {
    pub(super) user_agent: String,
    pub(super) user_agent_data: Option<UserAgentData>,
    pub(super) primary_language: Option<String>,
    pub(super) languages: Vec<String>,
    pub(super) time_zone: Option<String>,
    pub(super) time_zone_offset_minutes: i32,
    pub(super) screen: Screen,
    pub(super) hardware: Hardware,
    pub(super) webgl: Option<Webgl>,
    pub(super) canvas_hash: Option<String>,
    pub(super) install_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreparationReq {
    pub(super) app_version: String,
    pub(super) fingerprint: String,
    pub(super) extra_data: StableFingerprintData,
    // if you also want to accept ip in body (optional)
    pub(super) ip: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreparationResp {
    pub(super) ok: bool,
    pub(super) visitor_id: String,
}
