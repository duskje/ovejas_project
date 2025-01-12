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
    UpdateState(String),
    DestroyState,
}

impl From<Message> for RequestOperations {
 fn from(orig: Message) -> Self {
     let data = orig.into_data();
     let op_code = data.clone()[0];

     match op_code {
         0x1 => RequestOperations::RequestState,
         0x2 => {
             let state_payload = String::from_utf8(data.clone()[1..].into()).expect("Could not parse state payload");
             RequestOperations::UpdateState(state_payload)
         },
         0x3 => RequestOperations::DestroyState,
         _ => panic!("Unknown opcode {op_code}"),
     }
 }
}

impl From<RequestOperations> for Message {
    fn from(orig: RequestOperations) -> Self {
        let data: Vec<u8> = match orig {
            RequestOperations::RequestState => vec![0x1],
            RequestOperations::UpdateState(state_payload) => {
                let mut data = vec![0x2];
                data.append(&mut state_payload.into_bytes());
                data
            },
            RequestOperations::DestroyState => vec![0x3],
        };

        Message::Binary(data.into())
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
    pub project: String,
}

impl From<StateOperationMessage> for Message {
    fn from(orig: StateOperationMessage) -> Self {
        let serialized_state_op = serde_json::to_string(&orig)
            .expect("Could not convert to string");

        Message::Text(serialized_state_op.into())
    }
}
