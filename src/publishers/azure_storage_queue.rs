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
    pub message_flow_actions: HashMap<String, String>
}
#[async_trait]
impl Publisher<Arc<DefaultAzureCredential>> for AzureStorageQueue {
    type InitObj = QueueClient;
    type PubMessage = Message;

    fn flow_action_map(&self) -> &HashMap<String, String> {
        &self.message_flow_actions
    }
    fn repr(&self) -> String {
        format!("{}/{}", &self.storage_account, &self.queue_name)
    }
    fn init(&self, cred: Option<Arc<DefaultAzureCredential>>) -> QueueClient {
        let account = &self.storage_account;
        let queue_name = &self.queue_name;
        let storage_credentials = StorageCredentials::token_credential(
            cred.expect("Expected a DefaultAzureCredential but got None").clone()
        );
        let queue_service = QueueServiceClient::new(account, storage_credentials);
        queue_service.queue_client(queue_name)

    }
    async fn get_messages(&self, init_obj: &QueueClient) -> Vec<Message> {
        let response = init_obj.get_messages().await.unwrap();
        let raw_msgs = response.messages;
        raw_msgs
    }
    async fn task_done(&self, init_obj: &QueueClient, message: Self::PubMessage) {
        init_obj.pop_receipt_client(message).delete().await.expect(
            "Failed to mark task done"
        );
    }


}