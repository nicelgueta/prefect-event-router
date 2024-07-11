use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub trait LinkedPrefectJob {
    // returns flow_name, deployment_name
    fn get_flow_deployment(&self) -> (String, String);
}

#[derive(Debug)]
pub enum Error {
    PrefectApiError(String),
    InputError(String)
}
impl Error {
    pub fn to_string(&self) -> String {
        match self {
            Self::InputError(s) => format!("InputError: {}",s),
            Self::PrefectApiError(s) => format!("PrefectApiError: {}",s)
        }
    }
}

/// The Publisher trait defines the interface for different
/// configurations used to connect to a source
#[async_trait]
pub trait Publisher {
    type PubMessage: RawMessage;

    /// String representation of the queue-level identifer for the given config
    /// For example for an AzureStorageQueue -> "stroage-account-name/queue"
    fn repr(&self) -> String;

    /// mathod called to initialise client connections to the source
    /// prior to beginning the message loop
    async fn init(&mut self);

    /// Returns an iterator of messages
    async fn next_message(&mut self) -> Option<Self::PubMessage>;

    /// Mark a task as done if applicable. Just leave an empty implementation if not required
    async fn task_done(&mut self, message: Self::PubMessage);


}

pub trait RawMessage {
    fn get_content_str(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QMessage {
    flow_name: String,
    deployment_name: String,
    payload: Option<serde_json::Value>
}
impl QMessage {
    pub fn get_flow_parameters(&self) -> &Option<serde_json::Value> {
        &self.payload
    }
    pub fn get_flow_deployment(&self) -> (String, String) {
        (self.flow_name.clone(), self.deployment_name.clone())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::QMessage;
    

    #[test]
    fn test_load_q_message() {
        let json = json!(
            {
                "flow_name": "Test Flow",
                "deployment_name": "this-deployment",
                "payload": {"test_key": "test_value"}
            }
        );
        let q_message: QMessage = serde_json::from_value(json).expect("Failed to load json");
        assert_eq!(q_message.get_flow_deployment(), ("Test Flow".to_string(), "this-deployment".to_string()));
        assert_eq!(q_message.payload.expect(
            "Expected to find a payload as a serde json"
        )["test_key"], "test_value")
    }

    #[test]
    fn test_load_q_message_no_payload() {
        let json = json!(
            {
                "flow_name": "Test Flow",
                "deployment_name": "this-deployment",
            }
        );
        let q_message: QMessage = serde_json::from_value(json).expect("Failed to load json");
        assert_eq!(q_message.get_flow_deployment(), ("Test Flow".to_string(), "this-deployment".to_string()));
        assert_eq!(q_message.payload, None)
    }

    #[test]
    fn test_load_q_message_ignore_other_fields() {
        let json = json!(
            {
                "flow_name": "Test Flow",
                "deployment_name": "this-deployment",
                "some_random": "field"
            }
        );
        // will not compile if didn't work
        let q_message: QMessage = serde_json::from_value(json).expect("Failed to load json");
        assert_eq!(q_message.get_flow_deployment(), ("Test Flow".to_string(), "this-deployment".to_string()));
    }
}
