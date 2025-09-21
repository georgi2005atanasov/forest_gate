use serde::Serialize;

use crate::utils::error::{Error, Result};

#[derive(Clone)]
pub struct EmailClient {
    http: reqwest::Client,
    api_key: String,
    from_email: String,
    from_name: String,
    reply_to: Option<String>, // optional
}

impl EmailClient {
    pub fn from_env() -> Result<Self> {
        // you can use dotenvy::dotenv().ok(); if needed
        let api_key = std::env::var("SENDGRID_API_KEY").map_err(|_| missing_env("SENDGRID_API_KEY"))?;
        let from_email = std::env::var("FROM_EMAIL").map_err(|_| missing_env("FROM_EMAIL"))?;
        let from_name  = std::env::var("FROM_NAME").map_err(|_| missing_env("FROM_NAME"))?;
        let reply_to   = std::env::var("REPLY_TO_EMAIL").ok(); // optional

        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
            from_email,
            from_name,
            reply_to,
        })
    }

    pub async fn send_text_and_html(
        &self,
        to_email: &str,
        subject: &str,
        text: Option<&str>,
        html: Option<&str>,
    ) -> Result<()> {
        let url = "https://api.sendgrid.com/v3/mail/send";

        let mut content = Vec::new();
        if let Some(t) = text { content.push(SgContent { r#type: "text/plain".into(), value: t.into() }); }
        if let Some(h) = html { content.push(SgContent { r#type: "text/html".into(),  value: h.into() }); }
        if content.is_empty() {
            content.push(SgContent { r#type: "text/plain".into(), value: String::new() });
        }

        let body = SgMail {
            personalizations: vec![SgPersonalization {
                to: vec![SgEmail { email: to_email.into(), name: None }],
                subject: Some(subject.into()),
            }],
            from: SgEmail {
                email: self.from_email.clone(),
                name: Some(self.from_name.clone()),
            },
            reply_to: self.reply_to.as_ref().map(|e| SgEmail { email: e.clone(), name: None }),
            content,
        };

        let res = self.http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?; // uses From<reqwest::Error> for your Error

        // SendGrid success = 202 Accepted
        if res.status() == reqwest::StatusCode::ACCEPTED {
            tracing::info!("email sent!");
            Ok(())
        } else {
            let code = res.status().as_u16();
            let text = res.text().await.unwrap_or_default();
            Err(Error::Unexpected(format!("sendgrid failed: status={code} body={text}")))
        }
    }
}

fn missing_env(var: &'static str) -> Error {
    Error::Validation(format!("missing env var: {var}"))
}

#[derive(Serialize)]
struct SgMail {
    personalizations: Vec<SgPersonalization>,
    from: SgEmail,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<SgEmail>,
    content: Vec<SgContent>,
}

#[derive(Serialize)]
struct SgPersonalization {
    to: Vec<SgEmail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
}

#[derive(Serialize)]
struct SgEmail {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize)]
struct SgContent {
    #[serde(rename = "type")]
    r#type: String,
    value: String,
}
