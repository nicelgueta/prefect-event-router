use serde::{Deserialize, Serialize};

#[cfg(feature = "azure_storage_queues")]
mod azure_storage_queue;
// #[cfg(feature = "zmq")]
// mod zeromq;

mod stdin;

// enum to hold all the publishers that is used by serde to genrate from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "publisher_type")]
pub enum PublisherType {

    #[cfg(feature = "azure_storage_queues")]
    AzureStorageQueue(azure_storage_queue::AzureStorageQueue),
    // #[cfg(feature = "zmq")]
    // Zmq(zmq::Zmq),

    StdInput(stdin::StdInput)
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "azure_storage_queues")]
    use super::azure_storage_queue::AzureStorageQueue;
    use super::PublisherType;
    use serde_json::json;

    #[test]
    fn test_load_std_input(){
        let json_v = json!(
            {
                "publisher_type": "StdInput",
            }
        );
        let azure_publisher: PublisherType = serde_json::from_value(json_v).expect(
            "Unable to parse json as a valid publisher type"
        );
        let _stdin = match azure_publisher {
            PublisherType::StdInput(v) => v,
            _ => panic!("Not expecting any other type other than AzureStorageQueue")
        };

    }

    #[cfg(feature = "azure_storage_queues")]
    #[test]
    fn test_load_azure_publisher_type(){
        let json_v = json!(
            {
                "publisher_type": "AzureStorageQueue",
                "storage_account": "storage-account-name",
                "queue_name": "test",
            }
        );
        let azure_publisher: PublisherType = serde_json::from_value(json_v).expect(
            "Unable to parse json as a valid publisher type"
        );
        let _asq: AzureStorageQueue = match azure_publisher {
            PublisherType::AzureStorageQueue(v) => v,
            _ => panic!("Not expecting any other type other than AzureStorageQueue")
        };

    }
}
