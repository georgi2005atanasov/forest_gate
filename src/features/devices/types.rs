use serde::{Deserialize, Serialize};

use crate::features::{
    devices::DeviceType,
    onboarding::types::{PreparationReq, StableFingerprintData},
};

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

impl CreateDeviceDto {
    pub fn from_preparation(req: &PreparationReq) -> Self {
        let ua = req.extra_data.user_agent_data.as_ref();
        let os_name = ua.and_then(|u| u.platform.clone());
        let os_version = ua.and_then(|u| u.platform_version.clone());

        let locale = req
            .extra_data
            .primary_language
            .clone()
            .or_else(|| req.extra_data.languages.get(0).cloned());

        let device_type = match ua.and_then(|u| u.mobile) {
            Some(true) => DeviceType::Mobile,
            Some(false) => DeviceType::Desktop,
            None => DeviceType::Unknown,
        };

        CreateDeviceDto {
            os_name,
            os_version,
            locale,
            device_type,
            app_version: Some(req.app_version.clone()),
            fingerprint: Some(req.fingerprint.clone()),
            extra_data: req.extra_data.clone(), // keep your JSON mapping consistent
        }
    }
}
