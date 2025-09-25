// src/features/auth/token_service.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::error::Error;
use crate::{features::system::ConfigService, utils::error::Result};

#[derive(Clone)]
pub struct TokenService {
    cfg: Arc<ConfigService>,
    enc_key: EncodingKey,
    dec_key: DecodingKey,
    issuer: String,
    audience: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // subject = user id
    pub uid: i64,    // user id (numeric)
    pub did: i64,    // device id
    pub jti: String, // unique id for token
    pub iat: i64,    // issued at (unix)
    pub exp: i64,    // expires at (unix)
    pub iss: String, // issuer
    pub aud: String, // audience
}

#[derive(Debug, Serialize)]
pub struct IssuedTokens {
    pub access_token: String,
    pub access_expires_at: i64,
    pub refresh_token: Option<String>,
    pub refresh_expires_at: Option<i64>,
}

impl TokenService {
    /// `ec_private_pem` must be a PKCS#8 or SEC1 EC private key (P-256 recommended).
    /// `ec_public_pem` must be the matching public key.
    pub fn new(
        cfg: Arc<ConfigService>,
        issuer: impl Into<String>,
        audience: impl Into<String>,
    ) -> Result<Self> {
        let enc_key = EncodingKey::from_ec_pem(&std::fs::read("scripts/ec_private_pkcs8.pem")?)?;
        let dec_key = DecodingKey::from_ec_pem(&std::fs::read("scripts/ec_public.pem")?)?;
        Ok(Self {
            cfg,
            enc_key,
            dec_key,
            issuer: issuer.into(),
            audience: audience.into(),
        })
    }

    /// Create access + optional refresh token based on ConfigDto flags and durations.
    pub async fn mint_tokens(&self, user_id: i64, device_id: i64) -> Result<IssuedTokens> {
        let cfg = self.cfg.get().await?; // hot config (Redis → DB)
        let now = Utc::now();

        // ----- Access token -----
        let access_exp = now + Duration::seconds(cfg.token_validity_seconds as i64);
        let access_claims = TokenClaims {
            sub: user_id.to_string(),
            uid: user_id,
            did: device_id,
            jti: Uuid::new_v4().to_string(),
            iat: now.timestamp(),
            exp: access_exp.timestamp(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = None; // set KID if you rotate keys
        let access_token = encode(&header, &access_claims, &self.enc_key)
            .map_err(|e| Error::Unexpected(format!("encode access token: {e}")))?;

        // ----- Refresh token (conditional) -----
        let (refresh_token, refresh_expires_at) = if cfg.allow_refresh_tokens {
            let refresh_exp = now + Duration::seconds(cfg.refresh_token_validity_seconds as i64);
            let refresh_claims = TokenClaims {
                sub: user_id.to_string(),
                uid: user_id,
                did: device_id,
                jti: Uuid::new_v4().to_string(),
                iat: now.timestamp(),
                exp: refresh_exp.timestamp(),
                iss: self.issuer.clone(),
                aud: self.audience.clone(),
            };

            let refresh = encode(&header, &refresh_claims, &self.enc_key)
                .map_err(|e| Error::Unexpected(format!("encode refresh token: {e}")))?;
            (Some(refresh), Some(refresh_exp.timestamp()))
        } else {
            (None, None)
        };

        Ok(IssuedTokens {
            access_token,
            access_expires_at: access_exp.timestamp(),
            refresh_token,
            refresh_expires_at,
        })
    }

    /// Optional: verifying (useful for tests or refresh endpoints).
    pub fn verify(&self, token: &str) -> Result<TokenClaims> {
        let mut val = Validation::new(Algorithm::ES256);
        val.set_audience(&[self.audience.clone()]);
        val.set_issuer(&[self.issuer.clone()]);
        let data =
            decode::<TokenClaims>(token, &self.dec_key, &val).map_err(|e| Error::Unauthorized)?;
        Ok(data.claims)
    }
}
