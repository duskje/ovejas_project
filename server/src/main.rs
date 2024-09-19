use std::net::{TcpListener, TcpStream};
use std::thread::{spawn, sleep};
use std::time::Duration;

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response}, 
    WebSocket,
    Message,
};

use serde_json::Value;
use std::collections::HashSet;
use std::fs;

struct Delta {
    missing_remote_keys: Vec<String>,
    missing_local_keys: Vec<String>,
    difference: Vec<String>,
}

fn listen(websocket: &mut WebSocket<TcpStream>) {
    websocket.send(Message::Binary(vec![0x10].into())).expect("Could not send hello");

    let msg = websocket.read().unwrap(); // bloqueo hasta que se reciba algo

    if !msg.is_binary() {
        panic!("Invalid read!");
    }

    let message_data = msg.into_data();
    let remote_json: Value = serde_json::from_slice(message_data.as_slice()).expect("Could not deserialize");

    let local_state = fs::read("test.json").expect("Could not open local state");
    let local_json: Value = serde_json::from_slice(local_state.as_slice()).expect("Could not deserialize");

    let mut delta: Vec<String> = Vec::new();

    for (key, value) in local_json.as_object().unwrap() {
        let remote_object = remote_json.as_object().unwrap();

        if value != &remote_object[key] {
            delta.push(key.to_string());
        }
    };

    let local_keys: HashSet<&String> = HashSet::from_iter(local_json.as_object().unwrap().keys());
    let remote_keys: HashSet<&String> = HashSet::from_iter(remote_json.as_object().unwrap().keys());

    println!("Missing keys local: {:?}", local_keys.difference(&remote_keys));
    println!("Missing keys remote: {:?}", remote_keys.difference(&local_keys));
    println!("Keys with different values: {:?}", delta);

    sleep(Duration::from_secs(1));
}

fn main() {
    env_logger::init();

    let server = TcpListener::bind("127.0.0.1:3012").unwrap();

    for stream in server.incoming(){
        // Este move es para que el spawn (que hace uso de hebras)
        // sea dueño todo ese bloque (necesario por garantias de ciclo de vida)
        spawn(move || {
            let callback = |req: &Request, mut response: Response| { // esto es un lambda en Rust
                println!("handshake");
                println!("request path: {}", req.uri().path());

                for (header, _value) in req.headers() {
                    println!("* {header}");
                }

                let headers = response.headers_mut();

                headers.append("Authorization", "mi autorizacion".parse().unwrap());

                Ok(response)
            };

            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap(); // recibe la
                                                                                // conexión con
                                                                                // headers, a
                                                                                // diferencia de
                                                                                // accept()
            loop {
                listen(&mut websocket);
            }
        });
    }
}
