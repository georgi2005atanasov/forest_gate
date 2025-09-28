use super::User;
use crate::features::users::{types::CreateUserDto, LoginMethod};
use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder};
use time::Date;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn all(
        &self,
        email_verified: Option<bool>,
        phone_number_verified: Option<bool>,
        login_method: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<User>, i64), sqlx::Error> {
        let mut qb = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                id,
                username,
                email,
                phone_number,
                is_email_verified,
                is_phone_verified,
                password_hash,
                salt,
                login_method,
                created_at,
                updated_at,
                deleted_at
            FROM users
            WHERE deleted_at IS NULL
            "#,
        );

        // ---- WHERE filters
        if let Some(v) = email_verified {
            qb.push(" AND is_email_verified = ").push_bind(v);
        }
        if let Some(v) = phone_number_verified {
            qb.push(" AND is_phone_verified = ").push_bind(v);
        }
        if let Some(lm) = login_method {
            // Compare against text (works well if column is a PG enum)
            qb.push(" AND login_method = ").push_bind(lm);
        }
        // if let Some(from) = created_from {
        //     // Compare date part only
        //     qb.push(" AND created_at::date >= ").push_bind(from);
        // }
        // if let Some(to_) = created_to {
        //     qb.push(" AND created_at::date <= ").push_bind(to_);
        // }

        // ---- ORDER & pagination
        let page_limit = limit.clamp(1, limit);
        let page_offset = offset.max(offset);

        qb.push(" ORDER BY created_at DESC ");
        qb.push(" LIMIT ").push_bind(page_limit);
        qb.push(" OFFSET ").push_bind(page_offset);

        // ---- Execute SELECT
        let users: Vec<User> = qb.build_query_as().fetch_all(&self.pool).await?;

        // ---- Count total with same filters
        let mut count_qb = QueryBuilder::<Postgres>::new(
            r#"SELECT COUNT(*)::BIGINT AS total FROM users WHERE deleted_at IS NULL"#,
        );

        // Repeat filters
        if let Some(v) = email_verified {
            count_qb.push(" AND is_email_verified = ").push_bind(v);
        }
        if let Some(v) = phone_number_verified {
            count_qb.push(" AND is_phone_verified = ").push_bind(v);
        }
        if let Some(ref lm) = login_method {
            count_qb.push(" AND login_method = ").push_bind(lm);
        }
        // if let Some(from) = created_from {
        //     count_qb.push(" AND created_at::date >= ").push_bind(from);
        // }
        // if let Some(to_) = created_to {
        //     count_qb.push(" AND created_at::date <= ").push_bind(to_);
        // }

        #[derive(sqlx::FromRow)]
        struct Row {
            total: i64,
        }

        let total = count_qb
            .build_query_as::<Row>()
            .fetch_one(&self.pool)
            .await?
            .total;

        Ok((users, total))
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
                login_method, 
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
            user_dto.login_method,
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
            SELECT
                id,
                username,
                email,
                phone_number,
                password_hash,
                salt,
                is_email_verified,
                is_phone_verified,
                login_method,
                created_at,
                updated_at,
                deleted_at
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
            SELECT
                id,
                username,
                email,
                phone_number,
                password_hash,
                salt,
                is_email_verified,
                is_phone_verified,
                login_method,
                created_at,
                updated_at,
                deleted_at
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
            SELECT
                id,
                username,
                email,
                phone_number,
                password_hash,
                salt,
                is_email_verified,
                is_phone_verified,
                login_method,
                created_at,
                updated_at,
                deleted_at
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
