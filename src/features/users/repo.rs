use super::{User};
use crate::features::users::{types::CreateUserDto, LoginMethod};
use chrono::Utc;
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_dto: CreateUserDto,
        password_hash: String,
        salt: Vec<u8>,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users 
            (
                username, 
                email, 
                phone_number, 
                password_hash, 
                salt, 
                is_email_verified, 
                is_phone_verified, 
                login_method, 
                created_at, 
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING 
                id, 
                username, 
                email, 
                phone_number, 
                password_hash, 
                salt, 
                is_email_verified, 
                is_phone_verified, 
                login_method as "login_method: LoginMethod", 
                created_at, 
                updated_at, 
                deleted_at
            "#,
            user_dto.username,
            user_dto.email,
            user_dto.phone_number,
            password_hash,
            salt,
            true,  // is_email_verified - every user is created after email verification
            false, // is_phone_verified
            user_dto.login_method as LoginMethod,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, phone_number, password_hash, salt, is_email_verified, is_phone_verified, login_method as "login_method: LoginMethod", created_at, updated_at, deleted_at
            FROM users 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, phone_number, password_hash, salt, is_email_verified, is_phone_verified, login_method as "login_method: LoginMethod", created_at, updated_at, deleted_at
            FROM users 
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, phone_number, password_hash, salt, is_email_verified, is_phone_verified, login_method as "login_method: LoginMethod", created_at, updated_at, deleted_at
            FROM users 
            WHERE username = $1 AND deleted_at IS NULL
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}
