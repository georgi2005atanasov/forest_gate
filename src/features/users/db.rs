use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "login_method", rename_all = "snake_case")]
pub enum LoginMethod {
    WithPassword,
    WithEmail,
    WithPhoneNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub password_hash: String,
    pub salt: Vec<u8>,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub login_method: LoginMethod,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}