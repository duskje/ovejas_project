use futures::{future, SinkExt, StreamExt, TryStreamExt};
use tokio::{
    time::{sleep, Duration},
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{
    accept_hdr_async,
    WebSocketStream,
};

use std::{
    borrow::BorrowMut,
    collections::HashMap,
    io::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use figment::{Figment, providers::{Format, Yaml, Env}};

use serde::Deserialize;
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response}, 
    WebSocket,
};

use serde_json::Value;
use std::fs;

use server::state::StateDelta;
use shared::{RequestOperations, StateOperationMessage, Transaction};


// fn listen(websocket: &mut WebSocketStream<TcpStream>, message_queue) {
//     match message_queue {
//         Operation::RequestState => {
//             websocket.send(Operation::RequestState.into()).expect("Could not request state");
// 
//             let remote_state = websocket.read().unwrap(); // bloqueo hasta que se reciba algo
// 
//             if !remote_state.is_binary() {
//                 panic!("Invalid read!");
//             }
// 
//             let message_data = remote_state.into_data();
//             let remote_json: Value = serde_json::from_slice(message_data.as_slice()).expect("Could not deserialize");
// 
//             let local_state = fs::read("test.json").expect("Could not open local state");
//             let local_json: Value = serde_json::from_slice(local_state.as_slice()).expect("Could not deserialize");
// 
//             let state_delta = StateDelta::from_json(local_json, remote_json);
// 
//             println!("Missing keys local: {:?}", state_delta.not_in_local); // Should push delete
//                                                                             // transaction
//             println!("Missing keys remote: {:?}", state_delta.not_in_remote); // Should push add
//                                                                               // transaction
//             println!("Keys with different values: {:?}", state_delta.value_not_equal);
// 
//             *current_state = Operation::ExecuteTransaction(Transaction::Update(state_delta.value_not_equal[0].to_string())); // Testing only
// 
//             sleep(Duration::from_secs(1));
//         },
//         Operation::ExecuteTransaction(transaction) => {
//             websocket.send(Operation::ExecuteTransaction(transaction.clone()).into()).unwrap();
//             *current_state = Operation::RequestState;
//         },
//     }
// }
// 
// 
// fn last_main() {
//     env_logger::init();
// 
//     let config: Config = Figment::new()
//         .merge(Yaml::file("config.yaml"))
//         .join(Env::raw().only(&["PORT", "ADDRESS"]))
//         .extract().unwrap();
// 
//     let address = config.address.unwrap_or("127.0.0.1".into());
//     let port = config.port.unwrap_or("3000".into());
//     
//     let full_address = format!("{address}:{port}");
//     println!("Listening at {full_address}");
//     let server = TcpListener::bind(full_address).unwrap();
// 
//     for stream in server.incoming(){
//         // Este move es para que el spawn (que hace uso de hebras)
//         // sea dueño todo ese bloque (necesario por garantias de ciclo de vida)
//         spawn(move || {
//             let callback = |req: &Request, mut response: Response| { // esto es un lambda en Rust
//                 println!("handshake");
//                 println!("request path: {}", req.uri().path());
// 
//                 for (header, _value) in req.headers() {
//                     println!("* {header}");
//                 }
// 
//                 let headers = response.headers_mut();
// 
//                 headers.append("Authorization", "mi autorizacion".parse().unwrap());
// 
//                 Ok(response)
//             };
// 
//             let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();
// 
//             let mut current_state = Operation::RequestState;
// 
//             loop {
//                 listen(&mut websocket, &mut current_state);
//             }
//         });
//     }
// }
//
#[derive(Deserialize)]
struct Config {
    port: Option<String>,
    address: Option<String>,
}

type MessageQueue = Arc<Mutex<HashMap<String, String>>>;

async fn listen_device(ws_stream: &mut WebSocketStream<TcpStream>, current_state: &mut RequestOperations) {
    match current_state {
        RequestOperations::RequestState => {
            ws_stream
                .send(RequestOperations::RequestState.into())
                .await
                .expect("Could not send message");
            
            let next_from_stream = ws_stream
                .next()
                .await
                .unwrap()
                .expect("Could not receive message");

            let message_data = next_from_stream
                .into_text()
                .unwrap();

            let remote_json: Value = serde_json::from_str(message_data.as_str()).expect("Could not deserialize");

            println!("{remote_json}");
        },
    }

    sleep(Duration::from_millis(5000)).await;
}

#[derive(Debug)]
enum ListenerType {
    Device,
    CLI,
    Error,
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    let mut listener_type = ListenerType::Error;

    let callback = |req: &Request, mut response: Response| {
        println!("Handshake");
        println!("request path: {}", req.uri().path());

        for (header, value) in req.headers() {
            println!("* {header}: {value:?}");
        }

        let headers = response.headers_mut();

        headers.append("Authorization", "mi autorizacion".parse().unwrap());

        let machine_type = req.headers()
            .get("machine-type")
            .expect("Client didn't set a value to header 'machine-type'")
            .to_str()
            .expect("Error while retrieving header 'machine-type'");

        match machine_type {
            "device" => {
                listener_type = ListenerType::Device;
            },
            "cli" => {
                listener_type = ListenerType::CLI;
            }
            _ => {
                panic!("Invalid value for 'machine-type' ({machine_type})")
            }
        }

        println!("Listener type set to '{listener_type:?}'");

        Ok(response)
    };

    let mut ws_stream = accept_hdr_async(raw_stream, callback).await.expect("Error during handshake");

    match listener_type {
        ListenerType::Device => {
            let mut current_state = RequestOperations::RequestState;

            loop {
                listen_device(&mut ws_stream, &mut current_state).await;
            }

        },
        ListenerType::CLI => {
            let next_from_stream = ws_stream
                .next()
                .await
                .unwrap()
                .expect("Could not receive message");

            let message_data = next_from_stream
                .into_text()
                .unwrap();

            let state_operation_message: StateOperationMessage = serde_json::from_str(message_data.as_str()).expect("Could not deserialize");

            println!("{state_operation_message:?}");
            
        },
        _ => {panic!("Listener type not implemented")}
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config: Config = Figment::new()
        .merge(Yaml::file("config.yaml"))
        .join(Env::raw().only(&["PORT", "ADDRESS"]))
        .extract().unwrap();

    let address = config.address.unwrap_or("127.0.0.1".into());
    let port = config.port.unwrap_or("9734".into());

    let full_address = format!("{address}:{port}");

    println!("Listening at {full_address}");

    let try_socket = TcpListener::bind(full_address).await;
    let listener = try_socket.expect("Failed to bind");
    
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
