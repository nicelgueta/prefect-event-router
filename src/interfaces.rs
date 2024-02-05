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

    
    /// Default implementation for getting the name of the flow and deployment
    /// to kick off for a given message type
    fn get_flow_deployment(
        &self, message_type: &str
    ) -> Result<(String, String), Error> {
        let msg_typ_result = self.flow_action_map().get(message_type);
        match msg_typ_result {
            Some(flow_dep_str) => {
                let split: Vec<&str> = flow_dep_str.split("/").collect();
                if split.len() != 2 {
                    Err(Error::InputError("Flow/deployment name combo should have format: <flow name>/<deployment name>".to_string()))
                } else {
                    let (flow_name, deployment_name) = (split[0], split[1]);
                    Ok((flow_name.to_string(), deployment_name.to_string()))
                }
            }
            None => Err(Error::InputError(format!("No flow/deployment configured for message type `{}`", message_type)))
        }
    }

    /// Must be implemented to return the map of the flow actions
    /// available to the relevant publisher. Usually implemented in the
    /// config file as `self.message_flow_actions`
    fn flow_action_map(&self) -> &HashMap<String, String>;

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
    message_type: String,
    payload: Option<serde_json::Value>
}
impl QMessage {
    pub fn get_flow_parameters(&self) -> &Option<serde_json::Value> {
        &self.payload
    }
    pub fn typ(&self) -> &String {
        &self.message_type
    }
}

#[cfg(test)]
mod tests {
    use super::QMessage;
    

    #[test]
    fn test_load_q_message() {
        let json = r#"
        {"message_type": "TestPubType","payload": {"test_key": "test_value"}}
        "#;
        let q_message: QMessage = serde_json::from_str(json).expect("Failed to load json");
        assert_eq!(q_message.typ(), "TestPubType");
        assert_eq!(q_message.payload.expect(
            "Expected to find a payload as a serde json"
        )["test_key"], "test_value")
    }

    #[test]
    fn test_load_q_message_no_payload() {
        let json = r#"
        {"message_type": "TestPubType"}
        "#;
        let q_message: QMessage = serde_json::from_str(json).expect("Failed to load json");
        assert_eq!(q_message.typ(), "TestPubType");
        assert_eq!(q_message.payload, None)
    }

    #[test]
    fn test_load_q_message_ignore_other_fields() {
        let json = r#"
        {"message_type": "TestPubType", "not_valid_field": 34}
        "#;
        // will not compile if didn't work
        let q_message: QMessage = serde_json::from_str(json).expect("Failed to load json");
        assert_eq!(q_message.typ(), "TestPubType");
    }
}
