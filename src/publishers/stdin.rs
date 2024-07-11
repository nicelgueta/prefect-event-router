
use crate::interfaces::{Publisher, RawMessage};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};


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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdInput ;

#[async_trait]
impl Publisher for StdInput {
    type PubMessage = StdInMsg;

    fn repr(&self) -> String {
        String::from("Stdin")
    }
    async fn init(&mut self){}

    async fn next_message(&mut self) -> Option<Self::PubMessage> {
        println!("Paste a message: ");
        let stdin = io::stdin();
        let mut lines = BufReader::new(stdin).lines();
        Some(StdInMsg::new(lines.next_line().await.unwrap().unwrap()))
    }
    async fn task_done(&mut self, _message: Self::PubMessage) {}
}
