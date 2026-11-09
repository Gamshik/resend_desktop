//! Platform secure storage for small secrets such as API keys.
//!
//! - Android: values are encrypted with an AES-GCM key that lives in the Android Keystore
//!   (see `android/`), and the ciphertext is kept in the app's private SharedPreferences.
//! - Windows / macOS / iOS / Linux: the `keyring` crate (Credential Manager, Keychain, Secret Service).
//!
//! The plugin registers no webview commands, so JavaScript can't read secrets through it.

use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

mod error;
pub use error::{Error, Result};

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
use android::SecureStore;

#[cfg(not(target_os = "android"))]
mod keyring_store;
#[cfg(not(target_os = "android"))]
use keyring_store::SecureStore;

/// Access the store from an `App`, `AppHandle` or window.
pub trait SecureStoreExt<R: Runtime> {
    fn secure_store(&self) -> &SecureStore<R>;
}

impl<R: Runtime, T: Manager<R>> SecureStoreExt<R> for T {
    fn secure_store(&self) -> &SecureStore<R> {
        self.state::<SecureStore<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("secure-store")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let store = android::init(app, api)?;
            #[cfg(not(target_os = "android"))]
            let store = keyring_store::init(app, api)?;
            app.manage(store);
            Ok(())
        })
        .build()
}
