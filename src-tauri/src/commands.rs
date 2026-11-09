//! Tauri commands: the only API the webview can call.
//! None of them ever returns the raw API key.

use crate::db::{self, HistoryItem, NewHistory};
use crate::error::{AppError, AppResult};
use crate::keystore;
use crate::resend::{self, Attachment, KeyCheck, SendRequest};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub http: Client,
}

impl AppState {
    fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        // A poisoned lock only means an earlier command panicked mid-query; the connection is still usable.
        self.db.lock().unwrap_or_else(|p| p.into_inner())
    }
}

const MAX_RECIPIENTS: usize = 50; // Resend's per-email limit for `to`
const MAX_ATTACHMENT_BYTES: usize = 40 * 1024 * 1024; // Resend's limit after base64 decoding

// ---------- API key ----------

#[derive(Serialize)]
pub struct KeyStatus {
    saved: bool,
    masked: Option<String>,
    access: Option<String>,
}

#[tauri::command]
pub fn key_status(state: State<'_, AppState>) -> AppResult<KeyStatus> {
    let key = keystore::get()?;
    let access = db::get_setting(&state.db(), "key_access")?;
    Ok(KeyStatus {
        saved: key.is_some(),
        masked: key.as_deref().map(keystore::mask),
        access,
    })
}

#[derive(Serialize)]
pub struct SaveKeyResult {
    masked: String,
    check: KeyCheck,
}

fn access_label(check: &KeyCheck) -> &'static str {
    match check {
        KeyCheck::Full { .. } => "full",
        KeyCheck::SendingOnly => "sending_only",
    }
}

/// Validates the key against Resend first, and only stores it if Resend accepts it.
#[tauri::command]
pub async fn save_key(state: State<'_, AppState>, key: String) -> AppResult<SaveKeyResult> {
    let key = key.trim().to_owned();
    keystore::validate_format(&key)?;
    let check = resend::check_key(&state.http, &key).await?;
    keystore::set(&key)?;
    db::set_setting(&state.db(), "key_access", access_label(&check))?;
    Ok(SaveKeyResult { masked: keystore::mask(&key), check })
}

#[tauri::command]
pub async fn check_key(state: State<'_, AppState>) -> AppResult<KeyCheck> {
    let key = keystore::require()?;
    let check = resend::check_key(&state.http, &key).await?;
    db::set_setting(&state.db(), "key_access", access_label(&check))?;
    Ok(check)
}

#[tauri::command]
pub fn delete_key(state: State<'_, AppState>) -> AppResult<()> {
    keystore::delete()?;
    state.db().execute("DELETE FROM settings WHERE key = 'key_access'", [])?;
    Ok(())
}

// ---------- sending ----------

#[derive(Deserialize)]
pub struct SendInput {
    from: String,
    to: Vec<String>,
    #[serde(default)]
    cc: Vec<String>,
    #[serde(default)]
    bcc: Vec<String>,
    #[serde(default)]
    reply_to: Vec<String>,
    subject: String,
    html: Option<String>,
    text: Option<String>,
    #[serde(default)]
    attachments: Vec<Attachment>,
}

#[derive(Serialize)]
pub struct SendOutcome {
    id: String,
    history_id: i64,
}

fn clean(list: Vec<String>) -> Vec<String> {
    list.into_iter().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect()
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

#[tauri::command]
pub async fn send_email(state: State<'_, AppState>, input: SendInput) -> AppResult<SendOutcome> {
    let from = input.from.trim().to_owned();
    let to = clean(input.to);
    let cc = clean(input.cc);
    let bcc = clean(input.bcc);
    let reply_to = clean(input.reply_to);
    let subject = input.subject.trim().to_owned();
    let html = non_empty(input.html);
    let text = non_empty(input.text);

    if from.is_empty() {
        return Err(AppError::Invalid("Add a sender (From) address.".into()));
    }
    if to.is_empty() {
        return Err(AppError::Invalid("Add at least one recipient.".into()));
    }
    if to.len() > MAX_RECIPIENTS {
        return Err(AppError::Invalid(format!("Resend allows at most {MAX_RECIPIENTS} recipients in To.")));
    }
    if subject.is_empty() {
        return Err(AppError::Invalid("Add a subject.".into()));
    }
    if html.is_none() && text.is_none() {
        return Err(AppError::Invalid("The message body is empty.".into()));
    }
    let attachment_bytes: usize = input.attachments.iter().map(|a| a.content.len() / 4 * 3).sum();
    if attachment_bytes > MAX_ATTACHMENT_BYTES {
        return Err(AppError::Invalid("Attachments are larger than Resend's 40 MB limit.".into()));
    }

    let key = keystore::require()?;
    let req = SendRequest {
        from: &from,
        to: &to,
        cc: &cc,
        bcc: &bcc,
        reply_to: &reply_to,
        subject: &subject,
        html: html.as_deref(),
        text: text.as_deref(),
        attachments: &input.attachments,
    };
    let result = resend::send(&state.http, &key, &req).await;

    // Log every attempt, successful or not. Attachment contents are not stored, only names.
    let names: Vec<String> = input.attachments.iter().map(|a| a.filename.clone()).collect();
    let err_msg = result.as_ref().err().map(|e| e.to_string());
    let conn = state.db();
    let history_id = db::insert_history(
        &conn,
        NewHistory {
            resend_id: result.as_ref().ok().map(String::as_str),
            from: &from,
            to: &to,
            cc: &cc,
            bcc: &bcc,
            subject: &subject,
            html: html.as_deref(),
            text: text.as_deref(),
            attachments: &names,
            status: if result.is_ok() { "sent" } else { "failed" },
            error: err_msg.as_deref(),
        },
    )?;
    let id = result?;
    db::set_setting(&conn, "last_from", &from)?;
    Ok(SendOutcome { id, history_id })
}

// ---------- history ----------

#[tauri::command]
pub fn list_history(state: State<'_, AppState>, limit: Option<u32>) -> AppResult<Vec<HistoryItem>> {
    db::list_history(&state.db(), limit.unwrap_or(200).min(1000))
}

#[tauri::command]
pub fn delete_history_item(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    db::delete_history(&state.db(), id)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    db::clear_history(&state.db())
}

// ---------- settings ----------

#[derive(Serialize)]
pub struct Settings {
    default_from: Option<String>,
    last_from: Option<String>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    let conn = state.db();
    Ok(Settings {
        default_from: db::get_setting(&conn, "default_from")?,
        last_from: db::get_setting(&conn, "last_from")?,
    })
}

#[tauri::command]
pub fn set_default_from(state: State<'_, AppState>, value: String) -> AppResult<()> {
    let conn = state.db();
    let value = value.trim();
    if value.is_empty() {
        conn.execute("DELETE FROM settings WHERE key = 'default_from'", [])?;
        Ok(())
    } else {
        db::set_setting(&conn, "default_from", value)
    }
}
