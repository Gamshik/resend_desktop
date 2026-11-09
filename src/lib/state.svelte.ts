import { api, type Access, type Domain, type HistoryItem, type KeyCheck } from "./api";

export type View = "compose" | "history" | "settings" | "guide";

/** A draft handed to Compose (e.g. "send again" from History). */
export type Draft = {
  from: string;
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  mode: "text" | "html";
  body: string;
};

class AppStore {
  view = $state<View>("compose");
  ready = $state(false);

  keySaved = $state(false);
  keyMasked = $state<string | null>(null);
  keyAccess = $state<Access | null>(null);
  /** Only known for full-access keys; sending-only keys can't list domains. */
  domains = $state<Domain[] | null>(null);

  defaultFrom = $state<string | null>(null);
  lastFrom = $state<string | null>(null);

  pendingDraft = $state<Draft | null>(null);

  verifiedDomains = $derived(this.domains?.filter((d) => d.status === "verified") ?? []);

  async init() {
    const [status, settings] = await Promise.all([api.keyStatus(), api.getSettings()]);
    this.keySaved = status.saved;
    this.keyMasked = status.masked;
    this.keyAccess = status.access;
    this.defaultFrom = settings.default_from;
    this.lastFrom = settings.last_from;
    this.view = status.saved ? "compose" : "guide";
    this.ready = true;
    if (status.saved && status.access === "full") {
      // Refresh the domain list in the background; failures just leave it empty.
      api.checkKey().then((c) => this.applyCheck(c)).catch(() => {});
    }
  }

  applyCheck(check: KeyCheck) {
    this.keyAccess = check.access;
    this.domains = check.access === "full" ? check.domains : null;
  }

  keyStored(masked: string, check: KeyCheck) {
    this.keySaved = true;
    this.keyMasked = masked;
    this.applyCheck(check);
  }

  keyRemoved() {
    this.keySaved = false;
    this.keyMasked = null;
    this.keyAccess = null;
    this.domains = null;
  }

  reuse(item: HistoryItem) {
    const isHtml = !!item.html;
    this.pendingDraft = {
      from: item.from,
      to: item.to,
      cc: item.cc,
      bcc: item.bcc,
      subject: item.subject,
      mode: isHtml ? "html" : "text",
      body: (isHtml ? item.html : item.text) ?? "",
    };
    this.view = "compose";
  }
}

export const app = new AppStore();
