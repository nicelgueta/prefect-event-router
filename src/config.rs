use crate::publishers::PublisherType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub prefect_use_msal_auth: Option<bool>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    threads: Vec<PublisherType>,
    settings: Settings
}
impl ConfigFile {
    pub fn iter(&self) -> std::slice::Iter<PublisherType> {
        self.threads.iter()
    }
    pub fn get_settings_ptr(&self) -> Arc<Settings> {
        // need to use Arc pointer as will be cloned across
        // multiple threads
        return Arc::new(self.settings.clone())

    }
}