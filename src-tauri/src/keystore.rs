//! The API key lives only in the OS credential store and in Rust memory.
//! It is never returned to the webview; the UI only ever sees a masked form.

use crate::error::{AppError, AppResult};
use keyring::Entry;

const SERVICE: &str = "com.gamshik.resenddesktop";
const ACCOUNT: &str = "resend-api-key";

fn entry() -> AppResult<Entry> {
    Ok(Entry::new(SERVICE, ACCOUNT)?)
}

pub fn get() -> AppResult<Option<String>> {
    match entry()?.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn require() -> AppResult<String> {
    get()?.ok_or(AppError::NoKey)
}

pub fn set(key: &str) -> AppResult<()> {
    Ok(entry()?.set_password(key)?)
}

pub fn delete() -> AppResult<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
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
