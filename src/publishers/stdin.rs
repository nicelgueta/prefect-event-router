
use crate::interfaces::{Publisher, RawMessage};
use std::collections::HashMap;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdInput {
    message_flow_actions: HashMap<String, String>
}
pub struct StdInMsg {
    msg: String
}
impl StdInMsg {
    fn new(msg: String) -> Self {
        Self {msg}
    }
}
impl RawMessage for StdInMsg {
    fn get_content_str(&self) -> String {
        self.msg.clone()
    }
}

#[async_trait]
impl Publisher<bool> for StdInput {
    type InitObj = bool;
    type PubMessage = StdInMsg;

    fn flow_action_map(&self) ->  &HashMap<String,String> {
        &self.message_flow_actions
    }
    fn repr(&self) -> String {
        String::from("Stdin")
    }
    fn init(&self, _cred: Option<bool>) -> bool {
        false
    }
    async fn get_messages(&self, _init_obj: &Self::InitObj) -> Vec<Self::PubMessage> {
        println!("Paste a message: ");
        let stdin = io::stdin();
        let mut lines = BufReader::new(stdin).lines();
        let mut vec = Vec::new();
        vec.push(StdInMsg::new(lines.next_line().await.unwrap().unwrap()));
        vec
    }
    async fn task_done(&self, _init_obj: &Self::InitObj, _message: Self::PubMessage) {

    }
}
