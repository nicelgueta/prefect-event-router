use crate::publishers::PublisherType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    threads: Vec<PublisherType>
}
impl ConfigFile {
    pub fn iter(&self) -> std::slice::Iter<PublisherType> {
        self.threads.iter()
    }
}