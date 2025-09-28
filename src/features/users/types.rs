use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::users::{User};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub login_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub login_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            phone_number: user.phone_number,
            is_email_verified: user.is_email_verified,
            is_phone_verified: user.is_phone_verified,
            login_method: user.login_method.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailsReq {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 8))]
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginReq {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct UserDevice {
    pub user_id: i64,
    pub device_id: i64,
    pub paired_at: DateTime<Utc>,
    pub is_primary: bool,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub login_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            phone_number: user.phone_number,
            is_email_verified: user.is_email_verified,
            is_phone_verified: user.is_phone_verified,
            login_method: user.login_method,
            created_at: user.created_at,
            updated_at: user.updated_at,
            deleted_at: user.deleted_at,
        }
    }
}
