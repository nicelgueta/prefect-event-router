use crate::interfaces::{Publisher, RawMessage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use futures::StreamExt;
use tmq::{subscribe, Context};

#[derive(Serialize, Deserialize)]
pub struct Zmq {
    pub tcp_uri: String,
    pub topic: String,
    pub message_flow_actions: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    socket: Option<subscribe::Subscribe>
}
pub struct ZmqMsg {msg: String}
impl RawMessage for ZmqMsg {
    fn get_content_str(&self) -> String {
        self.msg.clone()
    }
}

#[async_trait]
impl Publisher for Zmq {
    type PubMessage = ZmqMsg;

    fn flow_action_map(&self) -> &HashMap<String, String> {
        &self.message_flow_actions
    }
    fn repr(&self) -> String {
        format!("ZMQ {}", self.tcp_uri)
    }
    async fn init(&mut self) {
        let socket: subscribe::Subscribe = subscribe(&Context::new())
            .connect(self.tcp_uri.as_str())
            .unwrap()
            .subscribe(self.topic.as_bytes())
            .unwrap();
        self.socket = Some(socket)
    }
    async fn next_message(&mut self) -> Option<ZmqMsg> {
        let next_message = self.socket.as_mut().unwrap().next();
        let nxt = next_message.await.unwrap();
        match nxt {
            Ok(msg) => {
                let mut full_msg = String::new();
                let it = msg.iter().map(|x| x.as_str().unwrap());
                for st in it {
                    full_msg.push_str(st)
                };
                Some(ZmqMsg{msg: full_msg})
            },
            Err(e) => {
                println!("Got error {:?}", e);
                None
            }
        }

    }
    async fn task_done(&mut self, _message: Self::PubMessage) {}
}

// // TODO: following test hangs so need to fix
// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;
//     use crate::interfaces::Publisher;

//     use super::Zmq;
//     use serde_json::json;
//     use std::time::Duration;
//     use zmq::{Context, SocketType};
//     use rand::Rng;
//     use tokio::time::timeout;

//     fn generate_tcp_address() -> String {

//         let mut rng = rand::thread_rng();
//         let port = rng.gen_range(2000..65000);
//         format!("tcp://127.0.0.1:{}", port)
//     }

//     #[tokio::test]
//     async fn test_e2e() {
//         let address = generate_tcp_address();
//         let pub_sock = Context::new().socket(SocketType::PUB).unwrap();
//         pub_sock.bind(&address).unwrap();
//         let topic = "preftopic";
//         let data = json!(
//             {"message_type": "MyTestMsg", "payload": {"mytest": "data"}}
//         ).to_string();
//         let mut mfa : HashMap<String, String> = HashMap::new();
//         mfa.insert("MyTestMsg".to_string(), "test/test".to_string());
//         let mut tzmq = Zmq {
//             tcp_uri: address,
//             topic: String::from(topic),
//             message_flow_actions: mfa,
//             socket: None
//         };
//         let mut data_chcked = false;
//         for _ in 0usize..5 {
//             pub_sock.send_multipart(vec![topic.as_bytes(), data.as_bytes()], 0).unwrap();
//             tzmq.init().await;
//             assert_eq!(tzmq.get_flow_deployment("MyTestMsg").expect(
//                 "This should not result in error"
//             ), ("test".to_string(), "test".to_string()));
//             // let el_next_msg = timeout(Duration::from_millis(1000), tzmq.next_message()).await;
//             let next_message = tzmq.next_message().await;
//             match next_message {
//                 Some(v) => {
//                     assert_eq!(v.msg, "data");
//                     data_chcked = true
//                 },
//                 None => panic!("Should have returned a message")
//             };
//         }
//         if ! data_chcked {
//             panic!("Should not have completed loop without checking data")
//         }


//     }

// }
