use std::vec;

use crate::features::{
    admin::AllUsersDto,
    users::{types::UserDto, UserRepository},
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AdminService {
    user_repo: UserRepository,
}

impl AdminService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool),
        }
    }

    pub async fn all(&self, dto: AllUsersDto) -> Result<(Vec<UserDto>, i64), sqlx::Error> {
        let (users, total) = self
            .user_repo
            .all(
                dto.email_verified,
                dto.phone_number_verified,
                dto.login_method.as_deref(),
                dto.limit.unwrap_or(40),
                dto.offset.unwrap_or(0),
            )
            .await?;

        tracing::debug!("Loaded users: {:?}", users);
        tracing::debug!("aaaaa {}", total);

        if users.len() == 0 {
            return Ok((vec![], 0));
        }

        let user_dtos: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();

        Ok((user_dtos, total))
    }
}
