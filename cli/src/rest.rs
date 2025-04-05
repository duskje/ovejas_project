use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceDeleteDTO {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCreateDTO {
    pub name: String,
    pub machine_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreateDTO {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeleteDTO {
    pub name: String,
}
