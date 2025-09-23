use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::users::{LoginMethod, User};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub password: String,
    pub login_method: LoginMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub login_method: LoginMethod,
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
            login_method: user.login_method,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UserDetailsReq {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 8))]
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UserLoginReq {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(length(min = 8))]
    pub password: String,
}
