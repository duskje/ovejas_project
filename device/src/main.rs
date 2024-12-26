use std::{fs, net::TcpStream};
use device::state::StateDelta;
use http::Request;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

use shared::RequestOperations;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::io;

fn listen(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>){
    let msg = socket.read().expect("Error reading message");

    if !msg.is_binary() {
        panic!("Invalid read!");
    }

    let op_code: RequestOperations = msg.into();

    match op_code {
        RequestOperations::RequestState => {
            println!("Server requested current state.");

            let contents = fs::read_to_string("local_state.json").expect("could not open file");
            socket.send(Message::Binary(contents.into())).expect("Could not send state to remote");
        },
    }
}

use std::process::Command;

use serde::{de::Error, Deserialize, Serialize};
use serde_json::{Result, Value};

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

trait ResourceActions {
    fn create(&self);
    fn update(&self);
    fn delete(&self);
}

impl ResourceActions for User {
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

// #[derive(Serialize, Deserialize, Debug)]
// struct Resource<T> {
//     urn: String,
//     parameters: T,
// }

#[derive(Serialize, Deserialize, Debug)]
struct Resource {
    urn: String,
    parameters: Value,
}

impl Resource {
    fn apply(&self, action: Action, dry_run: bool) -> Result<()> {
        let urn_split: Vec<&str> = self.urn.split("::").collect();
        let [module, kind, name] = urn_split.try_into().expect("Invalid urn");

        let resource = match kind {
            "User" => serde_json::from_value::<User>(self.parameters.clone())?,
            _ => panic!["resource does not exist"],
        };

        if !dry_run {
            match action {
                Action::Create => resource.create(),
                Action::Delete => resource.delete(),
                Action::Update => resource.update(),
            };
        }

        Ok(())
    }
}



//fn main() -> Result<()> {
//    // let target_state = r#"
//    // {
//    //   "urn": "ovejas.system::User::user_0",
//    //   "parameters": {
//    //     "name": "user0",
//    //     "uid": 500,
//    //     "gid": 100
//    //   }
//    // }"#;
//
//    // let resource: Resource = serde_json::from_str(target_state)?;
//    // println!("urn: {}", resource.parameters);
//    //
//    // resource.apply()?;
//
//    let home_dir = env::home_dir().unwrap();
//    let home_dir = home_dir.to_str().unwrap();
//    let file = File::open(format!("{home_dir}/device_info_with_res.json"));
//    //let file = File::open(format!("{home_dir}/device_info.json"));
//
//    let mut local_json = String::new();
//    let _ = file.unwrap().read_to_string(&mut local_json);
//
//    println!("Local JSON:");
//    println!("{local_json}");
//
//    let local_json: Value = serde_json::from_str(local_json.as_str())?;
//
//    let mut lines = io::stdin();
//    let mut remote_json = String::new();
//    lines.read_to_string(&mut remote_json).expect("olvidaste pipear el ejemplo");
//
//    println!("Remote JSON:");
//    println!("{remote_json}");
//
//    let remote_json: Value = serde_json::from_str(remote_json.as_str())?;
//
//    let delta = StateDelta::from_json(local_json.as_object().unwrap()["resources"].clone(), remote_json.as_object().unwrap()["resources"].clone());
//
//    println!("Delta:");
//    println!("{delta:?}");
//
//    Ok(())
//}
//
fn main() {
    env_logger::init();

    let request = Request::builder()
        .uri("ws://localhost:9734/socket")
        .header("sec-websocket-key", "foo")
        .header("machine-type", "device")
        .header("upgrade", "websocket")
        .header("host", "example.com")
        .header("connection", "upgrade")
        .header("sec-websocket-version", 13)
        .body(())
        .unwrap();

    let (mut websocket, response) = connect(request).expect("Could not connect to the server");

    println!("Connected successfully to the server!");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");

    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    loop {
        listen(&mut websocket);
    }
}
