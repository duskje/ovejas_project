use std::{fs, net::TcpStream};
use device::state::StateDelta;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

use shared::Operation;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::io;

fn listen(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>){
    let msg = socket.read().expect("Error reading message");

    if !msg.is_binary() {
        panic!("Invalid read!");
    }

    let op_code: Operation = msg.into();

    match op_code {
        Operation::RequestState => {
            println!("Server requested current state.");

            let contents = fs::read_to_string("local_state.json").expect("could not open file");
            socket.send(Message::Binary(contents.into())).expect("Could not send state to remote");
        },
        Operation::ExecuteTransaction(transaction) => {
            println!("Execute transaction, {transaction:?}");
        },
    }
}

// fn main() {
//     env_logger::init();
// 
//     let (mut websocket, response) = connect("ws://localhost:3000/socket").expect("Could not connect to the server");
// 
//     println!("Connected successfully to the server!");
//     println!("HTTP status code: {}", response.status());
//     println!("Response headers:");
// 
//     for (header, _value) in response.headers() {
//         println!("* {header}");
//     }
// 
//     loop {
//         listen(&mut websocket);
//     }
// }
//

use std::process::Command;

use serde::{de::Error, Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    gid: u32,
    uid: u32,
}

trait ResourceActions {
    fn create(&self);
    fn update(&self, last_state: &Self);
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
    
    fn update(&self, target_state: &Self) {
        let result = Command::new("usermod")
            .args([
                "--uid", target_state.uid.to_string().as_str(),
                "--gid", target_state.gid.to_string().as_str(),
                "--login", target_state.name.to_string().as_str(),
                self.name.as_str(),
            ])
            .output()
            .expect("Failed to execute process");
    }

    fn delete(&self) {
        let result = Command::new("userdel")
            .args([self.name.as_str()])
            .output()
            .expect("Failed to execute process");
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
    fn apply(&self) -> Result<()> {
        let urn_split: Vec<&str> = self.urn.split("::").collect();
        let [module, kind, name] = urn_split.try_into().expect("Invalid urn");

        match kind {
            "User" => {
                let resource: User = serde_json::from_value(self.parameters.clone())?;
                println!("{resource:?}");
            },
            _ => {},
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    // let target_state = r#"
    // {
    //   "urn": "ovejas.system::User::user_0",
    //   "parameters": {
    //     "name": "user0",
    //     "uid": 500,
    //     "gid": 100
    //   }
    // }"#;

    // let resource: Resource = serde_json::from_str(target_state)?;
    // println!("urn: {}", resource.parameters);
    //
    // resource.apply()?;

    let home_dir = env::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap();
    let file = File::open(format!("{home_dir}/device_info_with_res.json"));
    //let file = File::open(format!("{home_dir}/device_info.json"));

    let mut local_json = String::new();
    let _ = file.unwrap().read_to_string(&mut local_json);

    println!("Local JSON:");
    println!("{local_json}");

    let local_json: Value = serde_json::from_str(local_json.as_str())?;

    let mut lines = io::stdin();
    let mut remote_json = String::new();
    lines.read_to_string(&mut remote_json).expect("olvidaste pipear el ejemplo");

    println!("Remote JSON:");
    println!("{remote_json}");

    let remote_json: Value = serde_json::from_str(remote_json.as_str())?;

    let delta = StateDelta::from_json(local_json.as_object().unwrap()["resources"].clone(), remote_json.as_object().unwrap()["resources"].clone());

    println!("Delta:");
    println!("{delta:?}");

    Ok(())
}
