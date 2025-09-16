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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub phone_number: Option<String>,
    #[validate(length(min = 8))]
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