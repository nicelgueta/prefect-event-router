use crate::publishers::PublisherType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::ops::Deref;

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

pub struct ArcCache<K, V> where K: Eq + Hash {
    inner: Arc<HashMap<K, V>>
}
impl<K, V> ArcCache<K, V> where K: Eq + Hash {
    pub fn new() -> Self {
        Self {
            // need to wrap in a mutex so that one of the shared owners
            // guaranatees a lock on the Arc before mutating its contents.
            inner: Arc::new(HashMap::new())
        }
    }
    pub fn get(&self, k: &K) -> Option<&V> {
        self.inner.get(k)
    }
}
impl <K,V> Deref for ArcCache<K, V> where K: Eq + Hash {
    type Target = HashMap<K, V>;
    
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
impl <K,V> Clone for ArcCache<K, V> where K: Eq + Hash {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}