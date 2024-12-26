use tokio_tungstenite::tungstenite::Message;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Transaction {
    Add(String),
    Delete(String),
    Update(String),
}

#[derive(Debug)]
pub enum RequestOperations {
    RequestState,
}

impl From<Message> for RequestOperations {
 fn from(orig: Message) -> Self {
     let data = orig.into_data();
     let op_code = data.clone()[0];

     match op_code {
         0x1 => RequestOperations::RequestState,
         _ => panic!("Unknown opcode {op_code}"),
     }
 }
}

impl From<RequestOperations> for Message {
    fn from(orig: RequestOperations) -> Self {
        let op_code: Vec<u8> = match orig {
            RequestOperations::RequestState => vec![0x1],
        };

        Message::Binary(op_code.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StateAction {
    Up,
    Down,
    Preview,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateOperationMessage {
    pub environment: String,
    pub action: StateAction,
    pub state: Option<String>,
}

impl From<StateOperationMessage> for Message {
    fn from(orig: StateOperationMessage) -> Self {
        let serialized_state_op = serde_json::to_string(&orig)
            .expect("Could not convert to string");

        Message::Text(serialized_state_op.into())
    }
}
