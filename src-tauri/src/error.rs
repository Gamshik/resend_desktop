use serde::{Serialize, Serializer};

/// Every error a command can return. It is serialized to a plain string for the frontend,
/// so nothing sensitive (like the API key) may ever be put into a message here.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("No API key saved yet. Add one in Settings.")]
    NoKey,
    #[error("That doesn't look like a Resend API key (it should start with \"re_\").")]
    BadKeyFormat,
    #[error("Resend rejected the API key. Check that you copied it fully and that it wasn't revoked.")]
    InvalidKey,
    #[error("Resend: {0}")]
    Resend(String),
    #[error("Too many requests. Resend allows a few requests per second; wait a moment and retry.")]
    RateLimited,
    #[error("Network error: {0}")]
    Network(String),
    #[error("Credential store error: {0}")]
    Keyring(String),
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("{0}")]
    Invalid(String),
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        AppError::Keyring(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        // reqwest errors can include the URL but never headers, so the key can't leak here.
        AppError::Network(e.without_url().to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
