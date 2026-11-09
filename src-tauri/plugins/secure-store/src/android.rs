use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

const PLUGIN_IDENTIFIER: &str = "com.gamshik.securestore";

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<SecureStore<R>> {
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "SecureStorePlugin")?;
    Ok(SecureStore(handle))
}

pub struct SecureStore<R: Runtime>(PluginHandle<R>);

#[derive(Serialize)]
struct KeyArgs<'a> {
    key: &'a str,
}

#[derive(Serialize)]
struct SetArgs<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Deserialize)]
struct GetResponse {
    #[serde(default)]
    value: Option<String>,
}

impl<R: Runtime> SecureStore<R> {
    pub fn get(&self, key: &str) -> crate::Result<Option<String>> {
        let res: GetResponse = self.0.run_mobile_plugin("get", KeyArgs { key })?;
        Ok(res.value)
    }

    pub fn set(&self, key: &str, value: &str) -> crate::Result<()> {
        Ok(self.0.run_mobile_plugin("set", SetArgs { key, value })?)
    }

    pub fn delete(&self, key: &str) -> crate::Result<()> {
        Ok(self.0.run_mobile_plugin("delete", KeyArgs { key })?)
    }
}
