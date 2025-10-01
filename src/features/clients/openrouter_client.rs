use serde::{Deserialize, Serialize};

use crate::utils::error::{Error, Result};

#[derive(Clone)]
pub struct OpenRouterClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,              // default: https://openrouter.ai/api/v1
    app_name: Option<String>,      // optional: sent as X-Title
    default_model: Option<String>, // optional: OPENROUTER_MODEL
}

impl OpenRouterClient {
    /// Create client from env.
    /// Required:  OPENROUTER_API_KEY
    /// Optional:  OPENROUTER_BASE_URL (default https://openrouter.ai/api/v1)
    ///            OPENROUTER_APP_NAME  -> X-Title header
    ///            OPENROUTER_MODEL     -> default model to use
    pub fn from_env() -> Result<Self> {
        // dotenvy::dotenv().ok();
        let api_key =
            std::env::var("OPENROUTER_API_KEY").map_err(|_| missing_env("OPENROUTER_API_KEY"))?;
        let base_url = std::env::var("OPENROUTER_BASE_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
        let app_name = std::env::var("OPENROUTER_APP_NAME").ok();
        let default_model = std::env::var("OPENROUTER_MODEL").ok();

        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
            base_url,
            app_name,
            default_model,
        })
    }

    /// Perform a chat completion and return the full parsed response.
    pub async fn chat(
        &self,
        messages: &[OrMessage],
        model: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<OrChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let model = model
            .map(|s| s.to_string())
            .or_else(|| self.default_model.clone())
            .ok_or_else(|| {
                Error::Validation("missing model (pass arg or set OPENROUTER_MODEL)".into())
            })?;

        let body = OrChatRequest {
            model,
            messages: messages.to_vec(),
            temperature,
            max_tokens,
        };

        let mut req = self.http.post(&url).bearer_auth(&self.api_key).json(&body);

        if let Some(t) = &self.app_name {
            req = req.header("X-Title", t);
        }

        let resp = req.send().await?; // assumes From<reqwest::Error> for your Error

        if resp.status().is_success() {
            let parsed = resp.json::<OrChatResponse>().await?;
            Ok(parsed)
        } else {
            let code = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            Err(Error::Unexpected(format!(
                "openrouter failed: status={code} body={text}"
            )))
        }
    }

    /// Convenience helper: returns only the assistant text (first choice).
    pub async fn chat_text(
        &self,
        messages: &[OrMessage],
        model: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let res = self.chat(messages, model, temperature, max_tokens).await?;
        let content = res
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| Error::Unexpected("no choices returned".into()))?
            .message
            .content;
        Ok(content)
    }
}

fn missing_env(var: &'static str) -> Error {
    Error::Validation(format!("missing .env var: {var}"))
}

//
// === Request/Response types ===
//

#[derive(Serialize, Deserialize, Clone)]
pub struct OrMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub content: String,
}

#[derive(Serialize)]
struct OrChatRequest {
    pub model: String,
    pub messages: Vec<OrMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Deserialize)]
pub struct OrChatResponse {
    pub id: Option<String>,
    pub model: String,
    pub choices: Vec<OrChoice>,
    #[serde(default)]
    pub usage: Option<OrUsage>,
    pub created: Option<u64>,
}

#[derive(Deserialize)]
pub struct OrChoice {
    pub index: Option<u32>,
    pub message: OrMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct OrUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}
