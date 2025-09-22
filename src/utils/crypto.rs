use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// A small helper for HMAC signing/verification (SHA-256).
/// - `sign()` returns Base64 URL-safe (no padding).
/// - `verify()` does constant-time verification.
/// - `encode_cookie_value()` packs `id.sig`.
/// - `decode_cookie_value()` returns `Some(id)` only if signature is valid.
pub struct ClientHMAC {
    key: Vec<u8>,
}

impl ClientHMAC {
    /// Create from raw key bytes (recommended: 32 random bytes).
    pub fn new(key: &[u8]) -> Self {
        Self { key: key.to_vec() }
    }

    /// Create from hex-encoded key (e.g., from env: `openssl rand -hex 32`).
    pub fn from_hex_key(hex_key: &str) -> Result<Self, hex::FromHexError> {
        let key = hex::decode(hex_key)?;
        Ok(Self::new(&key))
    }

    /// HMAC-SHA256 over `input`, Base64 URL-safe (no padding).
    pub fn sign(&self, input: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.key).expect("HMAC key");
        mac.update(input.as_bytes());
        let tag = mac.finalize().into_bytes();
        URL_SAFE_NO_PAD.encode(tag)
    }

    /// Verify `signature_b64` is a valid HMAC for `input`.
    pub fn verify(&self, input: &str, signature_b64: &str) -> bool {
        let sig = match URL_SAFE_NO_PAD.decode(signature_b64) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let mut mac = HmacSha256::new_from_slice(&self.key).expect("HMAC key");
        mac.update(input.as_bytes());
        mac.verify_slice(&sig).is_ok()
    }

    /// Encode cookie value as `id.sig`.
    pub fn encode_cookie_value(&self, visitor_id: &str) -> String {
        let sig = self.sign(visitor_id);
        format!("{visitor_id}.{sig}")
    }

    /// Decode cookie value. Returns `Some(visitor_id)` only if signature is valid.
    pub fn decode_cookie_value(&self, raw: &str) -> Option<String> {
        if let Some((id, sig)) = raw.rsplit_once('.') {
            if self.verify(id, sig) {
                return Some(id.to_string());
            }
        }
        None
    }
}
