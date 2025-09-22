use actix_web::cookie::{Cookie, SameSite};
use chrono::Datelike;
use deadpool_redis::{redis::AsyncCommands, Pool};
use password_hash::rand_core::{OsRng, RngCore};
use sqlx::PgPool;
use time::Duration;
use uuid::Uuid;

use crate::{
    features::{
        clients::EmailClient,
        devices::{types::CreateDeviceDto, Device, DeviceRepository},
        onboarding::types::PreparationReq,
    },
    utils::{
        crypto::ClientHMAC,
        error::{Error, Result},
    },
};

pub const COOKIE_VISITOR: &str = "__Host-visitor_id";
pub const COOKIE_WITH_EMAIL: &str = "__Host-with_email";

#[derive(Clone)]
pub struct OnboardingService {
    pub hmac_client: ClientHMAC,
    pub redis_pool: Pool,
    pub device_repo: DeviceRepository,
}

impl OnboardingService {
    pub fn new(hmac_client: ClientHMAC, pool: PgPool, redis_pool: Pool) -> Self {
        Self {
            hmac_client,
            redis_pool,
            device_repo: DeviceRepository::new(pool),
        }
    }

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

    pub(super) fn has_visitor_cookie(&self, cookie: Option<Cookie<'static>>) -> Option<String> {
        if let Some(c) = cookie {
            if let Some(id) = self.hmac_client.decode_cookie_value(c.value()) {
                return Some(id);
            }
        }

        // the cookie was not set
        None
    }

    pub(super) async fn check_visitor_cookie_existence(&self, key: &str) -> Result<bool> {
        let mut conn = self.redis_pool.get().await.map_err(Error::from)?;

        // `EXISTS` returns i32 (0 = does not exist, 1 = exists)
        let exists: i32 = conn.exists(key).await.map_err(Error::from)?;
        Ok(exists > 0)
    }

    pub(super) fn read_or_set_visitor_cookie(
        &self,
        cookie: Option<Cookie<'static>>,
    ) -> (String, Option<Cookie<'static>>) {
        if let Some(c) = cookie {
            if let Some(id) = self.hmac_client.decode_cookie_value(c.value()) {
                return (id, None);
            }
            // invalid or tampered; fall through and re-issue
        }

        let new_id = Uuid::new_v4().to_string();
        let value = self.hmac_client.encode_cookie_value(&new_id);

        let cookie = Cookie::build(COOKIE_VISITOR, value)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::days(180))
            .path("/") // required for __Host- prefix (and do not set Domain)
            .finish();

        (new_id, Some(cookie))
    }

    pub(super) async fn send_otp(
        &self,
        user_email: &str,
        email_client: &EmailClient,
    ) -> Result<Cookie<'static>> {
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
        let redis_key = format!("otp:with_email:v1:{}", nonce);
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
        let cookie = Cookie::build(COOKIE_WITH_EMAIL, value)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::minutes(10)) // same as OTP TTL
            .path("/") // required for __Host-*
            .finish();

        Ok(cookie)
    }
}
