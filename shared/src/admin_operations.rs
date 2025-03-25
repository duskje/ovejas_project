use tokio_tungstenite::tungstenite::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum AdminUserAction {
    CreateUser,
    DeleteUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminUserOperationMessage {
    pub username: String,
    pub password: Option<String>,
}

impl From<AdminUserOperationMessage> for Message {
    fn from(orig: AdminUserOperationMessage) -> Self {
        let serialized_state_op = serde_json::to_string(&orig)
            .expect("Could not convert to string");

        Message::Text(serialized_state_op.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AdminDeviceAction {
    CreateDevice,
    DeleteDevice,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminDeviceOperationMessage {
    pub machine_id: String,
    pub token: String,
}

impl From<AdminDeviceOperationMessage> for Message {
    fn from(orig: AdminDeviceOperationMessage) -> Self {
        let serialized_state_op = serde_json::to_string(&orig)
            .expect("Could not convert to string");

        Message::Text(serialized_state_op.into())
    }
}
