//! SQLite holds only non-secret data: sent-email history and settings.

use crate::error::AppResult;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use std::path::Path;

pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         CREATE TABLE IF NOT EXISTS sent_emails (
             id          INTEGER PRIMARY KEY AUTOINCREMENT,
             resend_id   TEXT,
             from_addr   TEXT NOT NULL,
             to_addrs    TEXT NOT NULL,   -- JSON array
             cc_addrs    TEXT NOT NULL,   -- JSON array
             bcc_addrs   TEXT NOT NULL,   -- JSON array
             subject     TEXT NOT NULL,
             body_html   TEXT,
             body_text   TEXT,
             attachments TEXT NOT NULL,   -- JSON array of filenames
             status      TEXT NOT NULL,   -- 'sent' | 'failed'
             error       TEXT,
             created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         );
         CREATE TABLE IF NOT EXISTS settings (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );",
    )?;
    Ok(conn)
}

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub resend_id: Option<String>,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub html: Option<String>,
    pub text: Option<String>,
    pub attachments: Vec<String>,
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
}

pub struct NewHistory<'a> {
    pub resend_id: Option<&'a str>,
    pub from: &'a str,
    pub to: &'a [String],
    pub cc: &'a [String],
    pub bcc: &'a [String],
    pub subject: &'a str,
    pub html: Option<&'a str>,
    pub text: Option<&'a str>,
    pub attachments: &'a [String],
    pub status: &'a str,
    pub error: Option<&'a str>,
}

fn json(v: &[String]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".into())
}

fn unjson(s: String) -> Vec<String> {
    serde_json::from_str(&s).unwrap_or_default()
}

pub fn insert_history(conn: &Connection, h: NewHistory) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO sent_emails
           (resend_id, from_addr, to_addrs, cc_addrs, bcc_addrs, subject, body_html, body_text, attachments, status, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            h.resend_id, h.from, json(h.to), json(h.cc), json(h.bcc), h.subject,
            h.html, h.text, json(h.attachments), h.status, h.error
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_history(conn: &Connection, limit: u32) -> AppResult<Vec<HistoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, resend_id, from_addr, to_addrs, cc_addrs, bcc_addrs, subject, body_html, body_text,
                attachments, status, error, created_at
         FROM sent_emails ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |r| {
        Ok(HistoryItem {
            id: r.get(0)?,
            resend_id: r.get(1)?,
            from: r.get(2)?,
            to: unjson(r.get(3)?),
            cc: unjson(r.get(4)?),
            bcc: unjson(r.get(5)?),
            subject: r.get(6)?,
            html: r.get(7)?,
            text: r.get(8)?,
            attachments: unjson(r.get(9)?),
            status: r.get(10)?,
            error: r.get(11)?,
            created_at: r.get(12)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn delete_history(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM sent_emails WHERE id = ?1", [id])?;
    Ok(())
}

pub fn clear_history(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM sent_emails", [])?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}
