use serde::{Deserialize, Serialize};
use time::Date;



#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct AllUsersDto {
    /// Filter: is email verified
    pub email_verified: Option<bool>,
    /// Filter: is phone number verified
    pub phone_number_verified: Option<bool>,
    /// Filter by login method (stringified, e.g. "Password", "Google", etc.)
    pub login_method: Option<String>,
    /// Page size (1..=100). Default 20.
    pub limit: Option<i32>,
    /// Offset (>=0). Default 0.
    pub offset: Option<i32>,
}