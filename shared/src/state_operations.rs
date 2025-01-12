use tokio_tungstenite::tungstenite::Message;
use serde::{Deserialize, Serialize};

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
