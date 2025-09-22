use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{prelude::Type, FromRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "device_type_enum", rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Unknown,
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "device_status_enum", rename_all = "lowercase")]
pub enum DeviceStatus {
    Active,
    Inactive,
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
