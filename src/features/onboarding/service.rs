use actix_web::cookie::Cookie;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use chrono::Datelike;
use deadpool_redis::Pool;
use password_hash::rand_core::{OsRng, RngCore};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    features::{
        clients::EmailClient,
        devices::{types::CreateDeviceDto, Device, DeviceRepository},
        onboarding::types::PreparationReq,
        users::{types::CreateUserDto, LoginMethod, UserRepository},
    },
    utils::{
        crypto::ClientHMAC,
        error::{Error, Result},
    },
};

/// region Redis prefixes
pub const VISITOR_PREFIX: &str = "rl:prep:v1:visitor:";
pub const INSTALL_PREFIX: &str = "rl:prep:v1:install:";
pub const IP_PREFIX: &str = "rl:prep:v1:ip:";
pub const EMAIL_PREFIX: &str = "rl:email:";
pub const OTP_PREFIX: &str = "otp:with_email:v1:";
/// endregion Redis prefixes

#[derive(Clone)]
pub struct OnboardingService {
    hmac_client: ClientHMAC,
    redis_pool: Pool,
    device_repo: DeviceRepository,
    user_repo: UserRepository,
    pool: PgPool,
}

impl OnboardingService {
    pub fn new(hmac_client: ClientHMAC, pool: PgPool, redis_pool: Pool) -> Self {
        Self {
            hmac_client,
            redis_pool,
            device_repo: DeviceRepository::new(pool.clone()),
            user_repo: UserRepository::new(pool.clone()),
            pool: pool.clone(),
        }
    }

    pub(super) fn has_valid_cookie(&self, cookie: Option<Cookie<'static>>) -> Option<String> {
        if let Some(c) = cookie {
            if let Some(id) = self.hmac_client.decode_cookie_value(c.value()) {
                return Some(id);
            }
        }
        None
    }

    /// returning the device + cookie containing the id of the device
    pub async fn ensure_device_from_preparation(&self, req: &PreparationReq) -> Result<Device> {
        // 1) Try existing by fingerprint
        if let Some(existing) = self
            .device_repo
            .find_by_fingerprint(&req.fingerprint)
            .await?
        {
            return Ok(existing);
        }

        // 2) Map request -> DTO and create
        let dto = CreateDeviceDto::from_preparation(req);
        let created = self.device_repo.create(dto).await?;

        Ok(created)
    }

    /// returns visitor_id + cookie_value
    pub(super) fn read_or_set_visitor_cookie(
        &self,
        cookie_value: Option<&str>,
    ) -> (String, Option<String>) {
        if let Some(v) = cookie_value {
            if let Some(id) = self.hmac_client.decode_cookie_value(v) {
                return (id, None);
            }
            // invalid or tampered; fall through and re-issue
        }

        let new_id = Uuid::new_v4().to_string();
        let value = self.hmac_client.encode_cookie_value(&new_id);

        (new_id, Some(value))
    }

    /// TODO: Add a check whether the user is already verified
    pub(super) async fn verify_email(
        &self,
        email: &str,
        code: &str,
        cookie_value: Option<&str>,
    ) -> Result<String> {
        if let Some(_u) = self
            .user_repo
            .find_by_email(email)
            .await
            .map_err(Error::from)?
        {
            return Err(Error::UserAlreadyExists);
        }

        if let Some(v) = cookie_value {
            if let Some(nonce) = self.hmac_client.decode_cookie_value(v) {
                let mut conn = self.redis_pool.get().await.map_err(Error::from)?;
                let otp: Option<String> = deadpool_redis::redis::cmd("GET")
                    .arg(format!("{}{}", OTP_PREFIX, nonce))
                    .query_async(&mut conn)
                    .await
                    .map_err(Error::from)?;

                if let Some(otp) = otp {
                    if otp == code {
                        // the cookie is gonna store: email
                        let value = self.hmac_client.encode_cookie_value(&email);
                        return Ok(value);
                    }
                }
            }
        }

        return Err(Error::InvalidOtp("invalid or expired session".to_string()));
    }

    /// sends an email and generates a cookie with nonce in it -
    /// the nonce would be used to get the cookie value from Redis
    pub(super) async fn send_otp(
        &self,
        user_email: &str,
        email_client: &EmailClient,
    ) -> Result<String> {
        if let Some(_u) = self
            .user_repo
            .find_by_email(user_email)
            .await
            .map_err(Error::from)?
        {
            return Err(Error::UserAlreadyExists);
        }

        // 1) Generate nonce (32 bytes -> hex)
        let nonce = {
            let mut bytes = [0u8; 32];
            OsRng.fill_bytes(&mut bytes);
            hex::encode(bytes)
        };

        // 2) Generate 6-digit OTP (000000..999999), zero-padded
        let otp_num = (OsRng.next_u32() % 1_000_000) as u32;
        let otp_code = format!("{:06}", otp_num);

        // 3) Store OTP in Redis with TTL (e.g., 10 minutes)
        //    Key is bound to the nonce so only the user with the cookie can verify.
        let mut conn = self.redis_pool.get().await.map_err(Error::from)?;
        let redis_key = format!("{}{}", OTP_PREFIX, nonce);
        let ttl_seconds = 10 * 60; // 10 minutes
                                   // SET key value EX <ttl> NX  (only set if not exists)
        let _: () = deadpool_redis::redis::cmd("SET")
            .arg(&redis_key)
            .arg(&otp_code)
            .arg("EX")
            .arg(ttl_seconds)
            .arg("NX")
            .query_async(&mut conn)
            .await
            .map_err(Error::from)?;

        // 4) Build email (subject + text + html), include the OTP
        let subject = "Email Verification";

        let text_body = format!(
    "Your verification code is: {otp}\n\nThis code will expire in 10 minutes.\nIf you did not request this, you can ignore this email.",
    otp = otp_code
);

        let html_body = format!(
            r#"
<!doctype html>
<html>
  <body style="background:#f6f8fb;margin:0;padding:24px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;color:#0f172a;">
    <!-- ... -->
    <div style="text-align:center;margin:20px 0;">
      <div style="display:inline-block;letter-spacing:6px;font-weight:700;font-size:28px;color:#111827;background:#f3f4f6;border-radius:12px;padding:12px 18px;">
        {otp}
      </div>
    </div>
    <!-- ... -->
    © {year} Forest Gate
  </body>
</html>
"#,
            otp = otp_code,
            year = chrono::Utc::now().year()
        );

        email_client
            .send_text_and_html(
                user_email,
                subject,
                Some(text_body.as_str()),
                Some(html_body.as_str()),
            )
            .await?;

        // 6) Sign nonce and create cookie (__Host-with_email)
        let value = self.hmac_client.encode_cookie_value(&nonce);

        Ok(value)
    }

    /// returns user and device id
    pub(super) async fn ensure_user_with_device(
        &self,
        device_id: &i64,
        email: &str,
        password: &str,
        confirm_password: &str,
    ) -> Result<(i64, i64)> {
        // 1) basic checks
        if password != confirm_password {
            return Err(Error::InvalidOtp("passwords do not match".to_string()));
        }

        // 2) find or create the user
        let user = if let Some(_u) = self
            .user_repo
            .find_by_email(email)
            .await
            .map_err(Error::from)?
        {
            return Err(Error::UserAlreadyExists);
        } else {
            // Make a username from the email local part
            let username = email.split('@').next().unwrap_or(email).to_string();

            let user_dto = CreateUserDto {
                username,
                email: email.to_string(),
                phone_number: None,
                login_method: LoginMethod::WithPassword,
            };

            // Argon2 hashing
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|_| Error::InvalidOtp("failed to hash password".to_string()))?
                .to_string();

            // FIX: salt is a string; convert to bytes explicitly
            let salt_bytes: Vec<u8> = salt.as_str().as_bytes().to_vec();

            self.user_repo
                .create(user_dto, password_hash, salt_bytes)
                .await
                .map_err(Error::from)?
        };

        // 3) Is there an active device already?
        // FIX A (simplest): use fetch_one so you get a plain bool (not Option<bool>)
        let has_active_device = sqlx::query_scalar!(
            r#"
        SELECT EXISTS (
          SELECT 1 FROM user_devices
          WHERE user_id = $1 AND revoked_at IS NULL
        )
        "#,
            user.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::from)?;

        let has_active_device = has_active_device.unwrap_or(false);

        if has_active_device {
            println!("User already has an active device");
        } else {
            println!("No active device found, this will be the primary one");
        }

        let is_primary = Some(has_active_device);

        // 4) insert into user_devices, ignore if already linked
        sqlx::query!(
            r#"
        INSERT INTO user_devices (user_id, device_id, is_primary)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, device_id) DO NOTHING
        "#,
            user.id,
            device_id,
            is_primary
        )
        .execute(&self.pool)
        .await
        .map_err(Error::from)?;

        Ok((user.id, *device_id))
    }
}
