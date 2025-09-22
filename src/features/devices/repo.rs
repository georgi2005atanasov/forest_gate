use crate::features::onboarding::types::StableFingerprintData;
use chrono::Utc;
use sqlx::types::Json;
use sqlx::PgPool;

use super::{types::CreateDeviceDto, Device, DeviceStatus};

#[derive(Clone)]
pub struct DeviceRepository {
    pool: PgPool,
}

impl DeviceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dto: CreateDeviceDto) -> Result<Device, sqlx::Error> {
        let extra: serde_json::Value = serde_json::to_value(&dto.extra_data)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // then use `extra` in the INSERT
        let device = sqlx::query_as!(
    Device,
    r#"
    INSERT INTO devices
      (os_name, os_version, locale, device_type, device_status, app_version, fingerprint, extra_data, created_at)
    VALUES
      ($1,     $2,        $3,     $4,          $5,            $6,         $7,         $8,         $9)
    RETURNING
      id, os_name, os_version, locale,
      device_type as "device_type: _",
      device_status as "device_status: _",
      app_version, fingerprint,
      extra_data,                                  -- field type Option<JsonValue>
      created_at, deleted_at
    "#,
    dto.os_name,
    dto.os_version,
    dto.locale,
    dto.device_type as _,
    DeviceStatus::Active as _,
    dto.app_version,
    dto.fingerprint,
    extra,                                         // <- use the precomputed JSON value
    chrono::Utc::now()
)
.fetch_one(&self.pool)
.await?;

        Ok(device)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Device>, sqlx::Error> {
        let device = sqlx::query_as!(
            Device,
            r#"
        SELECT
          id,
          os_name,
          os_version,
          locale,
          device_type as "device_type: _",
          device_status as "device_status: _",
          app_version,
          fingerprint,
          extra_data,           -- <== no cast
          created_at,
          deleted_at
        FROM devices
        WHERE id = $1 AND deleted_at IS NULL
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(device)
    }

    pub async fn find_by_fingerprint(&self, fp: &str) -> Result<Option<Device>, sqlx::Error> {
        let device = sqlx::query_as!(
            Device,
            r#"
        SELECT
          id,
          os_name,
          os_version,
          locale,
          device_type as "device_type: _",
          device_status as "device_status: _",
          app_version,
          fingerprint,
          extra_data,           -- <== no cast
          created_at,
          deleted_at
        FROM devices
        WHERE fingerprint = $1 AND deleted_at IS NULL
        "#,
            fp
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(device)
    }
}
