// Typed wrappers around the Rust commands in src-tauri/src/commands.rs.
// The raw API key only ever travels one way (into `saveKey`); nothing here returns it.
import { invoke } from "@tauri-apps/api/core";

export type Domain = { name: string; status: string };
export type KeyCheck = { access: "full"; domains: Domain[] } | { access: "sending_only" };
export type Access = "full" | "sending_only";

export type KeyStatus = { saved: boolean; masked: string | null; access: Access | null };
export type SaveKeyResult = { masked: string; check: KeyCheck };

export type Attachment = { filename: string; content: string };
export type SendInput = {
  from: string;
  to: string[];
  cc: string[];
  bcc: string[];
  reply_to: string[];
  subject: string;
  html: string | null;
  text: string | null;
  attachments: Attachment[];
};
export type SendOutcome = { id: string; history_id: number };

export type HistoryItem = {
  id: number;
  resend_id: string | null;
  from: string;
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  html: string | null;
  text: string | null;
  attachments: string[];
  status: "sent" | "failed";
  error: string | null;
  created_at: string;
};

export type Settings = { default_from: string | null; last_from: string | null };

export const api = {
  keyStatus: () => invoke<KeyStatus>("key_status"),
  saveKey: (key: string) => invoke<SaveKeyResult>("save_key", { key }),
  checkKey: () => invoke<KeyCheck>("check_key"),
  deleteKey: () => invoke<void>("delete_key"),
  sendEmail: (input: SendInput) => invoke<SendOutcome>("send_email", { input }),
  listHistory: (limit?: number) => invoke<HistoryItem[]>("list_history", { limit }),
  deleteHistoryItem: (id: number) => invoke<void>("delete_history_item", { id }),
  clearHistory: () => invoke<void>("clear_history"),
  getSettings: () => invoke<Settings>("get_settings"),
  setDefaultFrom: (value: string) => invoke<void>("set_default_from", { value }),
};

/** Commands reject with the serialized AppError, which is a plain string. */
export function errorText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return "Something went wrong.";
}

export const EMAIL_RE = /^[^\s@<>]+@[^\s@<>]+\.[^\s@<>]+$/;

/** Accepts `a@b.com` or `Name <a@b.com>`. */
export function isAddress(v: string): boolean {
  const m = v.trim().match(/^(?:[^<>]*<)?([^<>]+)>?$/);
  return !!m && EMAIL_RE.test(m[1].trim());
}
