# Resend Desktop

A small app for sending email through [Resend](https://resend.com), for desktop and Android. Browsers can't call
the Resend API directly (it would expose the API key), so this app does it from a local Rust backend instead.

Built with Tauri 2, Rust and SvelteKit (Svelte 5). The code is written to also work on iOS, but iOS builds are not
tested.

## How the key is protected

- The API key is kept in **platform secure storage** by a small local plugin (`src-tauri/plugins/secure-store`), not
  in SQLite or a file:
  - **Android:** encrypted with AES-256-GCM using a key that is generated inside the Android Keystore and never leaves
    it. Only the ciphertext is written to the app's private storage.
  - **Windows / macOS / iOS / Linux:** Credential Manager, Keychain or Secret Service, via the `keyring` crate.
- The key **never reaches the webview**. The UI calls Rust commands (`send_email`, `check_key`, …); Rust reads the key,
  calls Resend, and returns only the result. The UI only ever sees a masked form like `re_••••••3fA9`. The
  secure-store plugin exposes no webview commands at all.
- Email HTML previews render in a fully sandboxed `<iframe sandbox="">` (no scripts, no same-origin access), and the
  app runs under a strict CSP (`src-tauri/tauri.conf.json`).
- SQLite (in the app data folder) holds only non-secret data: sent-email history and settings. Attachment contents are
  not stored, only file names.

The setup guide recommends a **Sending access** key: if it leaks, it can't read mail or manage the account. A
side effect is that such keys can't list domains, so the domain list only appears with a full-access key.

## Prerequisites

Desktop (Windows):

- Node.js and pnpm
- Rust: `winget install Rustlang.Rustup`
- Microsoft C++ Build Tools (the "Desktop development with C++" workload)
- WebView2 (preinstalled on Windows 11)

Android, in addition:

- Android Studio, with the Android SDK, SDK Platform-Tools, Build-Tools and the **NDK (Side by side)** installed from
  its SDK Manager
- Environment variables: `JAVA_HOME` (Android Studio's bundled JBR works, e.g.
  `C:\Program Files\Android\Android Studio\jbr`), `ANDROID_HOME` (e.g. `%LOCALAPPDATA%\Android\Sdk`) and `NDK_HOME`
  (`%ANDROID_HOME%\ndk\<version>`)
- Rust targets:
  `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`

See <https://tauri.app/start/prerequisites/> for macOS, Linux and iOS.

## Run

```bash
pnpm install
pnpm tauri dev      # run the desktop app
pnpm tauri build    # build an installer into src-tauri/target/release/bundle
```

Android:

```bash
pnpm tauri android init     # once: generates src-tauri/gen/android
pnpm tauri android dev      # run on a connected phone (USB debugging on) or an emulator
pnpm tauri android build --apk
```

`pnpm dev` alone opens the UI in a normal browser with a fake in-memory backend (`src/lib/dev-mock.ts`), which is
handy for design work. A key containing `full` acts as a full-access key there; a subject containing `fail` simulates a
failed send. The mock is never included in production builds. Narrow the window below 720px to see the phone layout.

## Layout

```
src-tauri/src/
  commands.rs   Tauri commands (the only API the UI can call)
  keystore.rs   API key access through the secure-store plugin
  resend.rs     Resend HTTP client (check key, send email)
  db.rs         SQLite: history + settings
  error.rs      Error type, serialized as plain strings for the UI
src-tauri/plugins/secure-store/
  src/          Rust side: keyring backend (desktop/iOS), Kotlin bridge (Android)
  android/      Kotlin plugin using the Android Keystore
src/
  lib/api.ts             Typed wrappers for the commands
  lib/state.svelte.ts    App state (Svelte 5 runes) and the phone-layout breakpoint
  lib/components/        Guide, Compose, History, Settings, …
```
