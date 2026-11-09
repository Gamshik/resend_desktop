# Resend Desktop

A small desktop app for sending email through [Resend](https://resend.com). Browsers can't call the Resend API
directly (it would expose the API key), so this app does it from a local Rust backend instead.

Built with Tauri 2, Rust and SvelteKit (Svelte 5).

## How the key is protected

- The API key is stored in the **OS credential store** (Windows Credential Manager, macOS Keychain, Linux Secret
  Service) via the `keyring` crate, not in SQLite or a file.
- The key **never reaches the webview**. The UI calls Rust commands (`send_email`, `check_key`, …); Rust reads the key,
  calls Resend, and returns only the result. The UI only ever sees a masked form like `re_••••••3fA9`.
- Email HTML previews render in a fully sandboxed `<iframe sandbox="">` (no scripts, no same-origin access), and the
  app runs under a strict CSP (`src-tauri/tauri.conf.json`).
- SQLite (in the app data folder) holds only non-secret data: sent-email history and settings. Attachment contents are
  not stored, only file names.

The setup guide recommends a **Sending access** key: if it leaks, it can't read mail or manage the account. A
side effect is that such keys can't list domains, so the domain list only appears with a full-access key.

## Prerequisites (Windows)

- Node.js and pnpm
- Rust: `winget install Rustlang.Rustup`
- Microsoft C++ Build Tools (the "Desktop development with C++" workload)
- WebView2 (preinstalled on Windows 11)

See <https://tauri.app/start/prerequisites/> for macOS and Linux.

## Run

```bash
pnpm install
pnpm tauri dev      # run the desktop app
pnpm tauri build    # build an installer into src-tauri/target/release/bundle
```

`pnpm dev` alone opens the UI in a normal browser with a fake in-memory backend (`src/lib/dev-mock.ts`), which is
handy for design work. A key containing `full` acts as a full-access key there; a subject containing `fail` simulates a
failed send. The mock is never included in production builds.

## Layout

```
src-tauri/src/
  commands.rs   Tauri commands (the only API the UI can call)
  keystore.rs   API key in the OS credential store
  resend.rs     Resend HTTP client (check key, send email)
  db.rs         SQLite: history + settings
  error.rs      Error type, serialized as plain strings for the UI
src/
  lib/api.ts             Typed wrappers for the commands
  lib/state.svelte.ts    App state (Svelte 5 runes)
  lib/components/        Guide, Compose, History, Settings, …
```
