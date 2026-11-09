use keyring::Entry;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use tauri::{AppHandle, Runtime, plugin::PluginApi};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<SecureStore<R>> {
    Ok(SecureStore {
        // Entries are grouped under the app identifier, e.g. "com.example.app".
        service: app.config().identifier.clone(),
        _runtime: PhantomData,
    })
}

pub struct SecureStore<R: Runtime> {
    service: String,
    _runtime: PhantomData<fn() -> R>,
}

impl<R: Runtime> SecureStore<R> {
    fn entry(&self, key: &str) -> crate::Result<Entry> {
        Ok(Entry::new(&self.service, key)?)
    }

    pub fn get(&self, key: &str) -> crate::Result<Option<String>> {
        match self.entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> crate::Result<()> {
        Ok(self.entry(key)?.set_password(value)?)
    }

    pub fn delete(&self, key: &str) -> crate::Result<()> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
