use serde::{Deserialize, Serialize};

mod azure_storage_queue;
mod stdin;

// enum to hold all the publishers that is used by serde to genrate from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "publisher_type")]
pub enum PublisherType {
    AzureStorageQueue(azure_storage_queue::AzureStorageQueue),
    StdInput(stdin::StdInput)
}

#[cfg(test)]
mod tests {

    use super::{PublisherType, azure_storage_queue::AzureStorageQueue};
    use serde_json::json;

    #[test]
    fn test_load_publisher_type(){
        let json_v = json!(
            {
                "publisher_type": "AzureStorageQueue",
                "storage_account": "storage-account-name",
                "queue_name": "test",
                "message_flow_actions": {
                    "MyMessageType": "Flow Name/deployment-name"
                }
            }
        );
        let azure_publisher: PublisherType = serde_json::from_value(json_v).expect(
            "Unable to parse json as a valid publisher type"
        );
        let azure_storage_config: AzureStorageQueue = match azure_publisher {
            PublisherType::AzureStorageQueue(v) => v,
            PublisherType::StdInput(v) => panic!("Not expecting stdinput type")
        };

        
    }
}
