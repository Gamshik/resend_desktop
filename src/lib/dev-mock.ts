// Dev-only fake backend so the UI can be worked on in a normal browser (`pnpm dev`)
// without the Rust side. Only loaded when not running inside Tauri; never part of a production build.
import { mockIPC } from "@tauri-apps/api/mocks";
import type { HistoryItem } from "./api";

export function installDevMock() {
  let key: string | null = null;
  const settings: Record<string, string> = {};
  const history: HistoryItem[] = [];
  let nextId = 1;
  const domains = [
    { name: "mail.example.com", status: "verified" },
    { name: "news.example.com", status: "pending" },
  ];
  const mask = (k: string) => `re_••••••${k.slice(-4)}`;
  const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

  mockIPC(async (cmd, args) => {
    const a = args as Record<string, any>;
    switch (cmd) {
      case "key_status":
        return { saved: !!key, masked: key && mask(key), access: settings.key_access ?? null };
      case "save_key": {
        await delay(500);
        if (!String(a.key).startsWith("re_")) throw 'That doesn\'t look like a Resend API key (it should start with "re_").';
        key = String(a.key).trim();
        const full = key.includes("full");
        settings.key_access = full ? "full" : "sending_only";
        return { masked: mask(key), check: full ? { access: "full", domains } : { access: "sending_only" } };
      }
      case "check_key":
        await delay(400);
        if (!key) throw "No API key saved yet. Add one in Settings.";
        return settings.key_access === "full" ? { access: "full", domains } : { access: "sending_only" };
      case "delete_key":
        key = null;
        delete settings.key_access;
        return null;
      case "send_email": {
        await delay(700);
        const i = a.input;
        const fail = i.subject.toLowerCase().includes("fail");
        const item: HistoryItem = {
          id: nextId++,
          resend_id: fail ? null : crypto.randomUUID(),
          from: i.from,
          to: i.to,
          cc: i.cc,
          bcc: i.bcc,
          subject: i.subject,
          html: i.html,
          text: i.text,
          attachments: i.attachments.map((x: { filename: string }) => x.filename),
          status: fail ? "failed" : "sent",
          error: fail ? "Resend: The mail.example.org domain is not verified." : null,
          created_at: new Date().toISOString(),
        };
        history.unshift(item);
        if (fail) throw item.error;
        settings.last_from = i.from;
        return { id: item.resend_id, history_id: item.id };
      }
      case "list_history":
        return history;
      case "delete_history_item":
        history.splice(history.findIndex((h) => h.id === a.id), 1);
        return null;
      case "clear_history":
        history.length = 0;
        return null;
      case "get_settings":
        return { default_from: settings.default_from ?? null, last_from: settings.last_from ?? null };
      case "set_default_from":
        settings.default_from = a.value;
        return null;
      case "plugin:opener|open_url":
        window.open(a.url, "_blank");
        return null;
    }
    throw `mock: unknown command ${cmd}`;
  });
}
