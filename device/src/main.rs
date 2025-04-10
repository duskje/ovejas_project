use std::collections::HashMap;
use std::fs::create_dir;
use std::ops::Deref;
use std::{fs, net::TcpStream};
use chrono::{DateTime, NaiveDateTime, Utc};
use device::state::StateDelta;
use figment::{Figment, providers::{Format, Yaml, Env}};
use http::{Request, Response};
use md5::{Md5, Digest};
use tungstenite::handshake::machine;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use walkdir::WalkDir;
use std::env::home_dir;
use std::path::Path;
use regex::Regex;

use shared::request_operations::{CurrentStatusResponse, DeviceStatus, EnvironmentUpdate, EnvironmentUpdateOperation, RequestOperations};

#[derive(Deserialize, Serialize, Debug)]
struct ResourceSchema {
    urn: String,
    parameters: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct StateSchema {
    schema_version: i32,
    resources: Vec<ResourceSchema>,
}

const OVEJAS_DIR: &str = ".ovejas";

fn get_ovejas_root_dir() -> String {
    let home = home_dir().unwrap();

    format!("{}/{OVEJAS_DIR}", home.to_string_lossy())
}

fn process_environment_update_request(environment: String, environment_update: EnvironmentUpdate) {
    let dry_run = false;

    match environment_update.operation {
        EnvironmentUpdateOperation::Create => {
            let target_state = environment_update.state.expect("Failed to get environment state");
            let target_state_json: serde_json::Value = serde_json::from_str(target_state.clone().as_str()).unwrap();

            let resources = target_state_json
                .get("resources")
                .expect("Target state doesn't have the key 'resources'")
                .as_array()
                .unwrap();

            for resource in resources {
                let resource: Resource = serde_json::from_value(resource.clone()).unwrap();
                resource.create(dry_run);

                println!("res {resource:?}");
            }

            let ovejas_root_dir = get_ovejas_root_dir();
            let state_file_path = format!("{ovejas_root_dir}/state/state.{environment}.json");

            if !dry_run {
                fs::write(state_file_path, target_state.clone().as_str())
                .expect(format!("Failed to write statefile({ovejas_root_dir})").as_str());
            }
        },
        EnvironmentUpdateOperation::Update =>  {
            let target_state = environment_update.state.expect("Failed to get environment state");
            let target_state_json: serde_json::Value = serde_json::from_str(target_state.clone().as_str()).unwrap();

            let ovejas_root_dir = get_ovejas_root_dir();
            let state_file_path = format!("{ovejas_root_dir}/state/state.{environment}.json");

            let local_state = fs::read_to_string(state_file_path.clone()).expect("Failed to read local state file");
            let local_state_json: serde_json::Value = serde_json::from_str(local_state.as_str()).unwrap();

            let delta = StateDelta::from_json(
                local_state_json.as_object().unwrap()["resources"].clone(),
                target_state_json.as_object().unwrap()["resources"].clone(),
            );

            for resource in delta.resources_to_update {
                let resource: Resource = serde_json::from_value(resource.clone()).unwrap();
                resource.update(dry_run);
            }

            for resource in delta.resources_to_delete {
                let resource: Resource = serde_json::from_value(resource.clone()).unwrap();
                resource.delete(dry_run);
            }

            for resource in delta.resources_to_create {
                let resource: Resource = serde_json::from_value(resource.clone()).unwrap();
                resource.create(dry_run);
            }

            if !dry_run {
                fs::write(state_file_path, target_state.clone().as_str())
                    .expect(format!("Failed to write statefile({ovejas_root_dir})").as_str());
            }
        },
        EnvironmentUpdateOperation::Destroy => {
            let ovejas_root_dir = get_ovejas_root_dir();
            let state_file_path = format!("{ovejas_root_dir}/state/state.{environment}.json");

            let local_state = fs::read_to_string(state_file_path.clone()).expect("Failed to read local state file");
            let local_state_json: serde_json::Value = serde_json::from_str(local_state.as_str()).unwrap();

            let resources = local_state_json
                .get("resources")
                .expect("Target state doesn't have the key 'resources'")
                .as_array()
                .unwrap();

            for resource in resources {
                let resource: Resource = serde_json::from_value(resource.clone()).unwrap();
                resource.delete(dry_run);

                println!("res {resource:?}");
            }

            if !dry_run {
                fs::remove_file(state_file_path.clone()).expect("Failed to remove state file");
            }
        },
    }
}

fn get_state_hashes() -> HashMap<String, [u8; 16]> {
    let state_dir = get_ovejas_root_dir();

    let mut state_hashes = HashMap::new();

    for dir in WalkDir::new(state_dir).min_depth(1) {
        let dir_result = dir.unwrap();

        let file_name: &str = dir_result.path().file_name().unwrap().to_str().unwrap();
        let captures = Regex::new(r"^state.([A-Za-z]+).json$").unwrap().captures(file_name);

        match captures {
            Some(captures) => {
                let file_path = dir_result.path().to_str().unwrap();
                let local_state = fs::read_to_string(file_path)
                    .expect("Could not open local state");

                let environment = captures.get(1).unwrap().as_str();

                let mut hasher = Md5::new();
                hasher.update(local_state);
                let state_hash: [u8; 16] = hasher.finalize().into();

                state_hashes.insert((environment).to_string(), state_hash);
            },
            None => {
                continue;
            }
        }
    }

    state_hashes
}

use tracing::{info, instrument};
use tracing_subscriber;

fn listen(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
    let msg = socket.read()?;

    if !msg.is_binary() {
        panic!("Invalid read!");
    }

    let op_code: RequestOperations = msg.into();

    match op_code {
        RequestOperations::StatusRequest => {
            info!("Remote requested current state");

            let state_hashes = get_state_hashes();

            let current_status = CurrentStatusResponse {
                status: DeviceStatus::Idle,
                timestamp: Utc::now().naive_utc().to_string(),
                state_hashes,
            };

            socket.send(current_status.into())
                .expect("Could not send device status to remote");
        },
        RequestOperations::UpdateEnvironmentsRequest(environment_updates) => {
            for (environment, environment_update) in environment_updates {
                info!(
                    environment = environment.clone(),
                    state = environment_update.clone().state,
                );

                process_environment_update_request(
                    environment,
                    environment_update,
                );
            }
        },
    };

    Ok(())
}

use std::process::Command;

use serde::{de::Error, Deserialize, Serialize};
use serde_json::{Result as SerdeJsonResult, Value};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    gid: u32,
    uid: u32,
}

enum Action {
    Create,
    Update,
    Delete,
}

trait ResourceProvider {
    fn create(&self);
    fn update(&self);
    fn delete(&self);
}

impl ResourceProvider for User {
    fn create(&self) {
        let result = Command::new("useradd")
            .args([
                "--uid", self.uid.to_string().as_str(),
                "--gid", self.gid.to_string().as_str(),
                self.name.as_str(),
            ])
            .output()
            .expect("Failed to execute process");

        println!("{result:?}");
    }
    
    fn update(&self) {
        let result = Command::new("usermod")
            .args([
                "--uid", self.uid.to_string().as_str(),
                "--gid", self.gid.to_string().as_str(),
                "--login", self.name.to_string().as_str(),
                self.name.as_str(),
            ])
            .output()
            .expect("Failed to execute process");

        println!("{result:?}");
    }

    fn delete(&self) {
        let result = Command::new("userdel")
            .args([self.name.as_str()])
            .output()
            .expect("Failed to execute process");

        println!("{result:?}");
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Resource {
    urn: String,
    parameters: Value,
}

impl Resource {
    fn get_provider(&self) -> Box<dyn ResourceProvider> {
        let urn_split: Vec<&str> = self.urn.split("::").collect();
        let [provider_module, kind, resource_id] = urn_split.try_into()
            .expect("Invalid urn");

        let provider = match kind {
            "User" => serde_json::from_value::<User>(self.parameters.clone()).unwrap(),
            _ => panic!["resource does not exist"],
        };

        Box::new(provider)
    }
    
//    fn execute(&self, action: Action, dry_run: bool) -> SerdeJsonResult<()> {
//        let urn_split: Vec<&str> = self.urn.split("::").collect();
//        let [provider_module, kind, resource_id] = urn_split.try_into()
//            .expect("Invalid urn");
//
//        let resource = match kind {
//            "User" => serde_json::from_value::<User>(self.parameters.clone())?,
//            _ => panic!["resource does not exist"],
//        };
//        
//        if !dry_run {
//            match action {
//                Action::Create => resource.create(),
//                Action::Delete => resource.delete(),
//                Action::Update => resource.update(),
//            };
//        }
//
//        Ok(())
//    }

    fn create(&self, dry_run: bool) {
        if dry_run {
            return;
        }

        let provider = self.get_provider();
        provider.deref().create();
    }

    fn delete(&self, dry_run: bool) {
        if dry_run {
            return;
        }

        let provider = self.get_provider();
        provider.deref().delete();
    }

    fn update(&self, dry_run: bool) {
        if dry_run {
            return;
        }

        let provider = self.get_provider();
        provider.deref().update();
    }
}


#[derive(Deserialize)]
struct Config {
    port: Option<u64>,
    address: Option<String>,
    machine_id: Option<String>,
    device_token: Option<String>,
}

#[derive(Debug)]
struct ServerError {
    reason_given: String,
}

#[derive(Debug, Deserialize)]
struct ServerResponse {
    data: Option<serde_json::Value>,
    msg: Option<String>,
}

fn main() {
    let ovejas_root_dir = get_ovejas_root_dir();

    if !Path::new(ovejas_root_dir.as_str()).exists() {
        fs::create_dir(ovejas_root_dir.clone()).expect("Failed to create state dir");
    }

    let config: Config = Figment::new()
        .merge(Yaml::file(format!("{}/config.yaml", ovejas_root_dir.clone())))
        .join(Env::raw().only(&["PORT", "ADDRESS", "DATABASE_PATH", "MACHINE_ID", "DEVICE_TOKEN"]))
        .extract().unwrap();

    let address = config.address.unwrap_or("localhost".into());
    let port = config.port.unwrap_or(9734u64.into());
    let machine_id = config.machine_id.expect("machine_id not set");
    let device_token = config.device_token.expect("device_token not set");

    let state_dir = format!("{ovejas_root_dir}/state");

    if !Path::new(state_dir.as_str()).exists() {
        fs::create_dir(state_dir).expect("Failed to create state dir");
    }

    tracing_subscriber::fmt::init();

    let full_address = format!("{address}:{port}");

    let request = Request::builder()
        .uri(format!("ws://{full_address}/socket"))
        .header("sec-websocket-key", "foo")
        .header("upgrade", "websocket")
        .header("host", address)
        .header("connection", "upgrade")
        .header("machine-type", "device")
        .header("machine-id", machine_id)
        .header("authorization", device_token)
        .header("sec-websocket-version", 13)
        .body(())
        .unwrap();

    let (mut websocket, response) = connect(request).map_err(|e: tungstenite::Error| {
        match e {
            tungstenite::Error::Http(response) => {
                let response_body = response.body().clone().unwrap();
                let response_json: ServerResponse = serde_json::from_str(String::from_utf8(response_body).unwrap().as_str()).expect("Could not parse server response");

                ServerError { reason_given: response_json.msg.unwrap() }
            },
            _ => ServerError { reason_given: String::from("No reason given") }
        }
    }).expect("Could not connect to the server");

    info!("Connected successfully to the server!");
    info!("HTTP status code: {}", response.status());
    info!("Response headers:");

    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    loop {
        match listen(&mut websocket) {
            Ok(_) => {},
            Err(err) => {
                println!("{err:?}");
            }
        };
    }
}
