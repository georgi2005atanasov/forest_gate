use actix_web::ResponseError;
// src/features/users/user_service.rs
use actix_web::{http::header, HttpRequest, HttpResponse};
use sqlx::PgPool;
use std::net::IpAddr;
use std::sync::Arc;

use super::types::UserLoginReq;
use crate::features::clients::MaxMindClient;
use crate::features::system::ConfigService;
use crate::features::users::helpers::{
    host_cookie, log_login_attempt, verify_password, COOKIE_DEVICE_ID, COOKIE_USER_ID,
};
use crate::features::users::repo::UserRepository;
use crate::utils::error::{Error, Result};
use crate::utils::token_service::TokenService;

#[derive(Clone)]
pub struct UserService {
    pool: PgPool,
    user_repo: UserRepository,
    token_service: Arc<TokenService>,
    config_service: Arc<ConfigService>,
    maxmind: Arc<MaxMindClient>,
}

impl UserService {
    pub fn new(
        pool: PgPool,
        token_service: Arc<TokenService>,
        config_service: Arc<ConfigService>,
        maxmind: Arc<MaxMindClient>,
    ) -> Self {
        Self {
            pool: pool.clone(),
            user_repo: UserRepository::new(pool.clone()),
            token_service,
            config_service,
            maxmind,
        }
    }

    pub async fn login(
        &self,
        req: &HttpRequest,
        payload: &UserLoginReq,
    ) -> actix_web::Result<HttpResponse> {
        // 1) client IP
        let client_ip: Option<IpAddr> = req
            .connection_info()
            .realip_remote_addr()
            .and_then(|s| s.parse().ok());

        // 2) find user (email or username)
        let (login_id_label, login_id_value) = if let Some(ref email) = payload.email {
            ("email", email.to_string())
        } else if let Some(ref username) = payload.username {
            ("username", username.to_string())
        } else {
            return Ok(Error::Validation("email or username is required".into()).error_response());
        };

        let user_opt = if login_id_label == "email" {
            self.user_repo
                .find_by_email(&login_id_value)
                .await
                .map_err(Error::from)?
        } else {
            self.user_repo
                .find_by_username(&login_id_value)
                .await
                .map_err(Error::from)?
        };

        let user = match user_opt {
            Some(u) => u,
            None => {
                let _ = log_login_attempt(&self.pool, &self.maxmind, None, client_ip, false).await;
                return Ok(Error::Unauthorized.error_response());
            }
        };

        // 3) password
        let ok = verify_password(&user.password_hash, &payload.password)
            .map_err(|e| Error::Unexpected(format!("password verify error: {e}")))?;
        if !ok {
            let _ =
                log_login_attempt(&self.pool, &self.maxmind, Some(user.id), client_ip, false).await;
            return Ok(Error::Unauthorized.error_response());
        }

        // 4) device cookie
        let device_id_cookie = req.cookie(COOKIE_DEVICE_ID);
        let device_id: i64 = match device_id_cookie.and_then(|c| c.value().parse::<i64>().ok()) {
            Some(d) => d,
            None => {
                let _ =
                    log_login_attempt(&self.pool, &self.maxmind, Some(user.id), client_ip, false)
                        .await;
                return Ok(
                    Error::Validation("missing or invalid device cookie".into()).error_response()
                );
            }
        };

        // 5) ensure user_devices link
        sqlx::query!(
            r#"
            INSERT INTO user_devices (user_id, device_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, device_id) DO NOTHING
            "#,
            user.id,
            device_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::from)?;

        // 6) tokens
        let tokens = self
            .token_service
            .mint_tokens(user.id, device_id)
            .await
            .map_err(|e| Error::Unexpected(format!("mint tokens: {e}")))?;

        // 7) log success
        let _ = log_login_attempt(&self.pool, &self.maxmind, Some(user.id), client_ip, true).await;

        // 8) cookies + JSON
        let cfg = self.config_service.get().await.map_err(Error::from)?;
        let mut resp = HttpResponse::Ok();

        let user_id_cookie = host_cookie(
            COOKIE_USER_ID,
            user.id.to_string(),
            cfg.refresh_token_validity_seconds as i64,
            true,
        );
        resp.cookie(user_id_cookie);

        let access_cookie = host_cookie(
            "__Host-access_token",
            tokens.access_token.clone(),
            cfg.token_validity_seconds as i64,
            true,
        );
        resp.cookie(access_cookie);

        // If you want refresh cookie, just uncomment:
        if let Some(ref rt) = tokens.refresh_token {
            let refresh_cookie = host_cookie(
                "__Host-refresh_token",
                rt.clone(),
                cfg.refresh_token_validity_seconds as i64,
                true,
            );
            resp.cookie(refresh_cookie);
        }

        #[derive(serde::Serialize)]
        struct LoginResponse {
            user_id: i64,
            device_id: i64,
            access_token: String,
            access_expires_at: i64,
            refresh_token: Option<String>,
            refresh_expires_at: Option<i64>,
        }

        Ok(resp
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .json(LoginResponse {
                user_id: user.id,
                device_id,
                access_token: tokens.access_token,
                access_expires_at: tokens.access_expires_at,
                refresh_token: tokens.refresh_token,
                refresh_expires_at: tokens.refresh_expires_at,
            }))
    }
}
