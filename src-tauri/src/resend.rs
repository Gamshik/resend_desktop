//! Thin client for the parts of the Resend REST API this app uses.

use crate::error::{AppError, AppResult};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};

const API: &str = "https://api.resend.com";

pub fn client() -> Client {
    Client::builder()
        .user_agent(concat!("resend-desktop/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client")
}

#[derive(Debug, Deserialize)]
struct ApiError {
    #[serde(default)]
    name: String,
    #[serde(default)]
    message: String,
}

async fn read_error(resp: Response) -> AppError {
    let status = resp.status();
    let body: ApiError = resp.json().await.unwrap_or(ApiError {
        name: String::new(),
        message: format!("HTTP {status}"),
    });
    if status == StatusCode::TOO_MANY_REQUESTS {
        return AppError::RateLimited;
    }
    // Resend spells this one `invalid_api_Key`; match case-insensitively to be safe.
    let name = body.name.to_ascii_lowercase();
    if name == "missing_api_key" || name == "invalid_api_key" {
        return AppError::InvalidKey;
    }
    // Everything else (unverified domain, bad recipient, ...) has a useful message; pass it on.
    AppError::Resend(body.message)
}

// ---------- key check ----------

#[derive(Debug, Serialize, Clone)]
pub struct Domain {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "access", rename_all = "snake_case")]
pub enum KeyCheck {
    /// Full-access key: we could list domains.
    Full { domains: Vec<Domain> },
    /// Sending-only key: valid, but Resend won't let it list domains.
    SendingOnly,
}

#[derive(Deserialize)]
struct DomainList {
    data: Vec<DomainRaw>,
}

#[derive(Deserialize)]
struct DomainRaw {
    name: String,
    status: String,
}

/// Verifies a key by calling `GET /domains`.
/// A sending-only key gets `401 restricted_api_key`, which still proves the key is real.
pub async fn check_key(http: &Client, key: &str) -> AppResult<KeyCheck> {
    let resp = http.get(format!("{API}/domains")).bearer_auth(key).send().await?;
    if resp.status().is_success() {
        let list: DomainList = resp.json().await?;
        return Ok(KeyCheck::Full {
            domains: list
                .data
                .into_iter()
                .map(|d| Domain { name: d.name, status: d.status })
                .collect(),
        });
    }
    let status = resp.status();
    let body: ApiError = resp.json().await.unwrap_or(ApiError {
        name: String::new(),
        message: String::new(),
    });
    match body.name.as_str() {
        "restricted_api_key" => Ok(KeyCheck::SendingOnly),
        _ if status == StatusCode::TOO_MANY_REQUESTS => Err(AppError::RateLimited),
        _ if status.is_client_error() => Err(AppError::InvalidKey),
        _ => Err(AppError::Resend(if body.message.is_empty() {
            format!("HTTP {status}")
        } else {
            body.message
        })),
    }
}

// ---------- sending ----------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Attachment {
    pub filename: String,
    /// Base64-encoded file content.
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct SendRequest<'a> {
    pub from: &'a str,
    pub to: &'a [String],
    #[serde(skip_serializing_if = "is_empty")]
    pub cc: &'a [String],
    #[serde(skip_serializing_if = "is_empty")]
    pub bcc: &'a [String],
    #[serde(skip_serializing_if = "is_empty")]
    pub reply_to: &'a [String],
    pub subject: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
    #[serde(skip_serializing_if = "is_empty")]
    pub attachments: &'a [Attachment],
}

#[derive(Deserialize)]
struct SendResponse {
    id: String,
}

pub async fn send(http: &Client, key: &str, req: &SendRequest<'_>) -> AppResult<String> {
    let resp = http.post(format!("{API}/emails")).bearer_auth(key).json(req).send().await?;
    if resp.status().is_success() {
        Ok(resp.json::<SendResponse>().await?.id)
    } else {
        Err(read_error(resp).await)
    }
}

fn is_empty<T>(s: &&[T]) -> bool {
    s.is_empty()
}
