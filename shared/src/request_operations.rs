use std::collections::HashMap;

use chrono::NaiveDateTime;
use tokio_tungstenite::tungstenite::Message;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum EnvironmentUpdateOperation {
    Create,
    Update,
    Destroy,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentUpdate {
    pub state: Option<String>,
    pub operation: EnvironmentUpdateOperation,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestOperations {
    StatusRequest,
    UpdateEnvironmentsRequest(HashMap<String, EnvironmentUpdate>),
}

impl From<RequestOperations> for Message {
    fn from(orig: RequestOperations) -> Self {
        let serialized = bincode::serialize(&orig).expect("Could not serialize");
        Message::Binary(serialized.into())
    }
}

impl From<Message> for RequestOperations {
    fn from(orig: Message) -> Self {
        let data = orig.into_data();
        let deserialized: RequestOperations = bincode::deserialize(data.as_ref()).expect("Could not deserialize");

        deserialized
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DeviceStatus {
    Idle,
    InProgress,
    Ready,
    // RolledBack,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentStatusResponse {
    pub status: DeviceStatus,
    pub timestamp: String,
    pub state_hashes: HashMap<String, [u8; 16]>,
}

impl From<CurrentStatusResponse> for Message {
    fn from(orig: CurrentStatusResponse) -> Self {
        let serialized = bincode::serialize(&orig).expect("Could not serialize");
        Message::Binary(serialized.into())
    }
}

impl From<Message> for CurrentStatusResponse {
    fn from(orig: Message) -> Self {
        let data = orig.into_data();
        let deserialized: CurrentStatusResponse = bincode::deserialize(data.as_ref()).expect("Could not deserialize");

        deserialized
    }
}
