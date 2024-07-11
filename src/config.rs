use crate::publishers::PublisherType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "azure_storage_queues")]
use crate::msal;
#[cfg(feature = "azure_storage_queues")]
use azure_identity::DefaultAzureCredential;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub prefect_use_msal_auth: Option<bool>,

    #[cfg(feature = "azure_storage_queues")]
    #[serde(skip_deserializing, skip_serializing)]
    azure_msal_credentials: Option<(String, String, String, String)>
}
impl Settings {
    pub async fn init(&mut self) {
        self.azure_msal_credentials = match self.prefect_use_msal_auth {
            Some(flag) => {
                if flag {
                    let credential = Some(Arc::new(DefaultAzureCredential::default()));
                    Some(msal::get_token_credentials(&credential).await.expect(
                        "Unable to acquire MSAL credentials from environment"
                    ))
                } else {
                    None
                }
            },
            None => None
        }
    }
    #[cfg(feature = "azure_storage_queues")]
    pub fn get_azure_credentials(&self) -> Option<&(String, String, String, String)> {
        self.azure_msal_credentials.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    threads: Vec<PublisherType>,
    settings: Settings
}
impl ConfigFile {
    pub async fn init(&mut self) {
        self.settings.init().await;
    }
    pub fn iter(&self) -> std::slice::Iter<PublisherType> {
        self.threads.iter()
    }
    pub fn get_settings_ptr(&self) -> Arc<Settings> {
        // need to use Arc pointer as will be cloned across
        // multiple threads
        return Arc::new(self.settings.clone())

    }
}