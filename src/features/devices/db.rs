use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::types::Json;
use sqlx::FromRow;

use crate::features::onboarding::types::StableFingerprintData;


#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "device_type_enum")] // matches your DB enum
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Bot,
    Unknown,
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Unknown
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "device_status_enum")] // matches your DB enum
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Active,
    Blocked,
    Disabled,
}

impl Default for DeviceStatus {
    fn default() -> Self {
        DeviceStatus::Active
    }
}

#[derive(Debug, FromRow)]
pub struct Device {
    pub id: i64,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub locale: Option<String>,
    pub device_type: DeviceType,
    pub device_status: DeviceStatus,
    pub app_version: Option<String>,
    pub fingerprint: Option<String>,
    pub extra_data: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Data needed to create a device (DTO)
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDeviceDto {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub locale: Option<String>,
    pub device_type: DeviceType,
    pub app_version: Option<String>,
    pub fingerprint: Option<String>,
    pub extra_data: StableFingerprintData,
}
