//! The API key lives only in platform secure storage (see the `secure-store` plugin) and in Rust memory.
//! It is never returned to the webview; the UI only ever sees a masked form.

use crate::error::{AppError, AppResult};
use tauri::{AppHandle, Runtime};
use tauri_plugin_secure_store::SecureStoreExt;

const ENTRY: &str = "resend-api-key";

pub fn get<R: Runtime>(app: &AppHandle<R>) -> AppResult<Option<String>> {
    Ok(app.secure_store().get(ENTRY)?)
}

pub fn require<R: Runtime>(app: &AppHandle<R>) -> AppResult<String> {
    get(app)?.ok_or(AppError::NoKey)
}

pub fn set<R: Runtime>(app: &AppHandle<R>, key: &str) -> AppResult<()> {
    Ok(app.secure_store().set(ENTRY, key)?)
}

pub fn delete<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    Ok(app.secure_store().delete(ENTRY)?)
}

pub fn validate_format(key: &str) -> AppResult<()> {
    let ok = key.starts_with("re_")
        && key.len() >= 10
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if ok { Ok(()) } else { Err(AppError::BadKeyFormat) }
}

/// `re_••••••3fA9` — enough to recognise which key is saved, useless to an attacker.
pub fn mask(key: &str) -> String {
    let tail: String = key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("re_••••••{tail}")
}
