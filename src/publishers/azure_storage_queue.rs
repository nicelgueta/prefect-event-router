use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use azure_storage_queues::prelude::*;
use azure_storage::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use azure_identity::DefaultAzureCredential;
use azure_storage_queues::operations::Message;

use crate::interfaces::{Publisher, RawMessage};


impl RawMessage for Message {
    fn get_content_str(&self) -> String {
        self.message_text.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureStorageQueue {
    pub storage_account: String,
    pub queue_name: String, 
    pub message_flow_actions: HashMap<String, String>,

    #[serde(skip_serializing, skip_deserializing)]
    queue_client: Option<QueueClient>,
    #[serde(skip_serializing, skip_deserializing)]
    messages: Option<Vec<Message>>
}
#[async_trait]
impl Publisher for AzureStorageQueue {
    type PubMessage = Message;

    fn flow_action_map(&self) -> &HashMap<String, String> {
        &self.message_flow_actions
    }
    fn repr(&self) -> String {
        format!("{}/{}", &self.storage_account, &self.queue_name)
    }
    async fn init(&mut self) {
        let account = &self.storage_account;
        let queue_name = &self.queue_name;
        let credential = Arc::new(DefaultAzureCredential::default());
        let storage_credentials = StorageCredentials::token_credential(
            credential
        );
        let queue_service = QueueServiceClient::new(account, storage_credentials);
        let queue_client = queue_service.queue_client(queue_name);
        // put first messages on internal vec
        self.messages = Some(queue_client.get_messages().await.unwrap().messages);
        self.queue_client = Some(queue_client);

    }

    async fn next_message(&mut self) -> Message {
        match &self.messages{
            Some(msgs) => {
                if msgs.len() == 0 {
                    let response = self.queue_client.as_ref().expect(
                        "Cannot await messages without the QueueClient being initialised"
                    ).get_messages().await.unwrap();
                    self.messages = Some(response.messages);
                }
            },
            None => {
                let response = self.queue_client.as_ref().expect(
                    "Cannot await messages without the QueueClient being initialised"
                ).get_messages().await.unwrap();
                self.messages = Some(response.messages);
            }
        };
        self.messages.as_mut().expect("Expecting a vec but got None").pop().unwrap()
    }
    async fn task_done(&mut self, message: Self::PubMessage) {
        self.queue_client.as_ref().expect(
            "Cannot call task done on a message when QueueClient not initialised"
        ).pop_receipt_client(message).delete().await.expect(
            "Failed to mark task done"
        );
    }


}