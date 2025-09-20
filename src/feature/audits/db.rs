use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, Copy)]
#[sqlx(type_name = "event_type_enum", rename_all = "snake_case")]
pub enum EventType {
    // Admin related
    ConfigChange,

    // User related
    Login,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, Copy)]
#[sqlx(type_name = "log_level_enum", rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, FromRow, Clone)]
pub struct AuditEvent {
    pub id: Uuid,
    pub user_id: i64,
    pub event_type: EventType,
    pub log_level: LogLevel,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
