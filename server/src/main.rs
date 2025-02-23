use futures::{future, SinkExt, StreamExt, TryStreamExt};
use md5::{Md5, Digest};
use tokio::{
    time::{sleep, Duration},
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{
    accept_hdr_async, tungstenite::handshake::client::Request, WebSocketStream
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
    accept, accept_hdr, handshake::server::Response, http::StatusCode, WebSocket
};

use serde_json::Value;
use std::fs;

use server::{schema::{devices, environments, projects}, state::StateDelta};
use shared::request_operations::{CurrentStatusResponse, EnvironmentUpdate, EnvironmentUpdateOperation, RequestOperations};
use shared::state_operations::{StateOperationMessage, StateAction};



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


#[derive(Deserialize)]
struct Config {
    port: Option<String>,
    address: Option<String>,
    database_url: Option<String>,
}

type MessageQueue = Arc<Mutex<HashMap<String, String>>>;


async fn listen_device(
    session: &mut ListenerSession,
    current_state: &mut RequestOperations,
    database_pool: Pool,
) {
    match current_state {
        RequestOperations::StatusRequest => {
            session.ws_stream
                .send(RequestOperations::StatusRequest.into())
                .await
                .expect("Could not send message");
            
            let next_from_stream = session.ws_stream
                .next()
                .await
                .unwrap()
                .expect("Could not receive message");

            let status_request_response: CurrentStatusResponse = next_from_stream.into();
            let state_hashes = status_request_response.state_hashes.clone();

            let conn = database_pool.get().await.expect("Could not get database connection");
            
            let machine_id = session.machine_id.clone();

            let environments = conn.interact(move |conn| {
                let device = devices::table
                    .filter(devices::name.eq(machine_id))
                    .select(Devices::as_select())
                    .get_result(conn)
                    .optional()
                    .expect("Database error");

                println!("{device:?}");

                if device.is_none() {
                    panic!("Device not found in database, rejecting connection")
                }

                let environments: Vec<Environments> = DevicesEnvironments::belonging_to(&device.unwrap())
                    .inner_join(environments::table)
                    .select(Environments::as_select())
                    .load(conn)
                    .expect("Database error");

                environments
            }).await;

            let mut environments_to_update = HashMap::new();

            for environment in environments.unwrap() {
                println!("{environment:?}");

                let environment_name = environment.name.clone();

                let latest_state = conn.interact(move |conn| {
                    let latest_state: Option<States> = States::belonging_to(&environment)
                        .select(States::as_select())
                        .order(states::id.desc())
                        .limit(1)
                        .get_result(conn)
                        .optional()
                        .expect("Database error");

                    latest_state
                }).await;

                let latest_state_json = latest_state.unwrap().unwrap().json;

                let device_environment_hash = state_hashes
                    .get(&environment_name);

                match device_environment_hash {
                    Some(hash) => {
                        let mut hasher = Md5::new();
                        hasher.update(latest_state_json.clone());
                        let latest_state_hash: [u8; 16] = hasher.finalize().into();

                        println!("state_delta {:?}", &latest_state_hash == device_environment_hash.unwrap());

                        if &latest_state_hash != hash {
                            let environment_update = EnvironmentUpdate {
                                state: Some(latest_state_json.clone()),
                                operation: EnvironmentUpdateOperation::Update,
                            };

                            environments_to_update.insert(
                                environment_name,
                                environment_update,
                            );
                        }
                    },
                    None => {
                        println!("environment '{}' not found", environment_name);

                        let environment_update = EnvironmentUpdate {
                            state: Some(latest_state_json.clone()),
                            operation: EnvironmentUpdateOperation::Create,
                        };

                        environments_to_update.insert(
                            environment_name,
                            environment_update,
                        );
                    },
                }
            }

            session.ws_stream.send(RequestOperations::UpdateEnvironmentsRequest(environments_to_update).into())
                .await
                .expect("Failed to send update request");

//            let message_data = next_from_stream
//                .into_text()
//                .unwrap();
//
//            let status_request_reponse = 

            // let remote_json: Value = serde_json::from_str(message_data.as_str()).expect("Could not deserialize");

            println!("{status_request_response:?}");

            sleep(Duration::from_millis(5000)).await;
        },
        _ => panic!("Invalid request operation")
    }
}

#[derive(Debug)]
enum ListenerType {
    Device,
    CLI,
    Error,
}

use server::models::*;
use server::schema::*;

struct ListenerSession {
    machine_id: String,
    listener_type: ListenerType,
    bearer_token: String,
    ws_stream: WebSocketStream<TcpStream>,
}

async fn is_device_registered(machine_id: String, database_pool: Pool) -> bool { 
    let conn = database_pool.get().await.expect("Could not get database connection");

    let is_device_none = conn.interact(move |conn| {
        let device = devices::table
            .filter(devices::name.eq(machine_id.clone()))
            .select(Devices::as_select())
            .get_result(conn)
            .optional()
            .expect("Database error");

        return device.is_none();
    }).await.expect("Could not fetch device from database");

    return !is_device_none
}

async fn new_session(raw_stream: TcpStream, database_pool: Pool)->ListenerSession {
    let mut listener_type = ListenerType::Error;
    let mut machine_id: String = String::default();
    let mut bearer_token: String = String::default();

    let callback = |req: &Request, mut response: Response| {
        println!("request path: {}", req.uri().path());

        for (header, value) in req.headers() {
            println!("* {header}: {value:?}");
        }

        let auth_value = req.headers()
            .get("authorization");

        if auth_value.is_none() {
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            return Ok(response);
        }

        bearer_token = auth_value
            .expect("Client didn't set a value to header 'machine-id'")
            .to_str()
            .expect("Error while retrieving header 'machine-id'")
            .to_string();

        machine_id = req.headers()
            .get("machine-id")
            .expect("Client didn't set a value to header 'machine-id'")
            .to_str()
            .expect("Error while retrieving header 'machine-id'")
            .to_string();

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

    let ws_stream = accept_hdr_async(raw_stream, callback).await.expect("Error during handshake");

    ListenerSession {
        machine_id,
        listener_type,
        bearer_token,
        ws_stream,
    }
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr, database_pool: Pool) {
    let mut session = new_session(raw_stream, database_pool.clone()).await;

    let device_registered = is_device_registered(
        session.machine_id.clone(),
        database_pool.clone()
    ).await;

    if !device_registered {
        println!("Device not registered; closing connection...");
        return
    }

    // if !session.authenticated {
    //     println!("Authentication error; closing connection...");
    //     return;
    // }

    match session.listener_type {
        ListenerType::Device => {
            let mut current_state = RequestOperations::StatusRequest;

            loop {
                listen_device(&mut session, &mut current_state, database_pool.clone()).await;
            }
        },
        ListenerType::CLI => {
            let next_from_stream = session.ws_stream
                .next()
                .await
                .unwrap()
                .expect("Could not receive message");

            let message_data = next_from_stream
                .into_text()
                .unwrap();

            let state_operation_message: StateOperationMessage = serde_json::from_str(message_data.as_str()).expect("Could not deserialize");

            println!("Command: {:?}", state_operation_message.action);

            let conn = database_pool.get().await.expect("Could not get database connection");

            match state_operation_message.action {
                StateAction::Up => {
                    let _ = conn.interact(move |conn| {
                        let project_result = projects::table
                            .filter(projects::name.eq(state_operation_message.project.clone()))
                            .select(Projects::as_select())
                            .get_result(conn)
                            .optional()
                            .unwrap();

                        let project: Projects = match project_result {
                            Some(project) => {
                                println!("loaded project: {}", state_operation_message.project.clone());

                                project
                            },
                            None => {
                                println!("created project: {}", state_operation_message.project.clone());

                                insert_into(projects::dsl::projects)
                                    .values(projects::name.eq(state_operation_message.project.clone()))
                                    .get_result(conn)
                                    .unwrap()
                            }
                        };

                        let environment_result: Option<Environments> = Environments::belonging_to(&project)
                            .select(Environments::as_select())
                            .filter(environments::name.eq(state_operation_message.environment.clone()))
                            .get_result(conn)
                            .optional()
                            .unwrap();

                        let environment: Environments = match environment_result {
                            Some(environment) => {
                                println!("loaded environment: {}", state_operation_message.environment.clone());

                                environment
                            },
                            None => {
                                println!("created project: {}", state_operation_message.environment.clone());

                                insert_into(environments::dsl::environments)
                                    .values((
                                        environments::name.eq(state_operation_message.environment.clone()),
                                        environments::project_id.eq(project.id),
                                    ))
                                    .get_result(conn)
                                    .unwrap()
                            }
                        };

                        insert_into(states::dsl::states)
                            .values((
                                states::json.eq(state_operation_message.state.expect("Expected a JSON").clone()),
                                states::environment_id.eq(environment.id),
                            ))
                            .execute(conn).expect("Could not insert state");
                        }).await;
                },
                StateAction::Preview => {

                },
                StateAction::Admin => {

                },
                _ => unimplemented!("action not implemented yet")

            }
        },
        _ => {panic!("Listener type not implemented")}
    }
}

use server::models::*;
use diesel::{insert_into, prelude::*};
use deadpool_diesel::{sqlite::{Manager, Pool, Runtime}, InteractError};

#[tokio::main]
async fn main() {
    env_logger::init();

    let config: Config = Figment::new()
        .merge(Yaml::file("config.yml"))
        .join(Env::raw().only(&["PORT", "ADDRESS", "DATABASE_URL"]))
        .extract().unwrap();

    let database_url = config.database_url.expect("Database url is required.");
    let manager = Manager::new(database_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(8)
        .build()
        .unwrap();
    
    let address = config.address.unwrap_or("127.0.0.1".into());
    let port = config.port.unwrap_or("9734".into());

    let full_address = format!("{address}:{port}");

    println!("Listening at {full_address}");

    let try_socket = TcpListener::bind(full_address).await;
    let listener = try_socket.expect("Failed to bind");
    
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, pool.clone()));
    }
}
