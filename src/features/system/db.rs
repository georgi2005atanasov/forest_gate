#[derive(sqlx::FromRow)]
pub struct ConfigEntity {
    pub id: i32,
    pub allow_recovery_codes: bool,
    pub allow_refresh_tokens: bool,
    pub token_validity_seconds: i32,
    pub refresh_token_validity_seconds: i32,
    pub ai_model: String,
    pub vector_similarity_threshold: i32,
}
