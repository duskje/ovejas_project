use futures::{SinkExt, StreamExt};
use md5::{Md5, Digest};
use tokio::{
    time::{sleep, Duration},
    net::TcpListener,
};

use tokio_tungstenite::WebSocketStream;

use std::{collections::HashMap, convert::Infallible, net::SocketAddr, str::FromStr};

use figment::{Figment, providers::{Format, Yaml, Env}};

use serde::Deserialize;

use server::{controller::handle_http_connection, schema::{devices, environments, projects}};
use shared::request_operations::{CurrentStatusResponse, EnvironmentUpdate, EnvironmentUpdateOperation, RequestOperations};
use shared::state_operations::{StateOperationMessage, StateAction};
use serde_json::json;

use tracing::{info, debug, error, instrument};
use tracing_subscriber;

#[derive(Deserialize)]
struct Config {
    port: Option<u64>,
    address: Option<String>,
    database_url: Option<String>,
}

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
                    .filter(devices::machine_id.eq(machine_id))
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

                if latest_state_json == "{}" {
                    let environment_update = EnvironmentUpdate {
                        state: None,
                        operation: EnvironmentUpdateOperation::Destroy,
                    };

                    environments_to_update.insert(
                        environment_name,
                        environment_update,
                    );

                    continue;
                }

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
                        println!("Environment '{}' not found, sending state as is...", environment_name);

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
}

use server::models::*;
use server::schema::*;

struct ListenerSession {
    machine_id: String,
    listener_type: ListenerType,
    bearer_token: String,
    ws_stream: WebSocketStream<TokioIo<Upgraded>>,
}

async fn is_device_registered(machine_id: String, database_pool: Pool) -> bool { 
    let conn = database_pool.get().await.expect("Could not get database connection");

    let is_device_none = conn.interact(move |conn| {
        let device = devices::table
            .filter(devices::machine_id.eq(machine_id.clone()))
            .select(Devices::as_select())
            .get_result(conn)
            .optional()
            .expect("Database error");

        return device.is_none();
    }).await.expect("Could not fetch device from database");

    info!(device_registered = !is_device_none);

    return !is_device_none
}

fn is_http_connection(req: &mut Request<Incoming>) -> bool { 
    let upgrade = HeaderValue::from_static("Upgrade");
    let websocket = HeaderValue::from_static("websocket");

    let headers = req.headers();

    let key = headers.get(SEC_WEBSOCKET_KEY);

    let derived = key.map(|k| derive_accept_key(k.as_bytes()));

    return req.method() != Method::GET
        || req.version() < Version::HTTP_11
        || !headers
            .get(CONNECTION)
            .and_then(|h| h.to_str().ok())
            .map(|h| {
                h.split(|c| c == ' ' || c == ',')
                    .any(|p| p.eq_ignore_ascii_case(upgrade.to_str().unwrap()))
            })
            .unwrap_or(false)
        || !headers
            .get(UPGRADE)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false)
        || !headers.get(SEC_WEBSOCKET_VERSION).map(|h| h == "13").unwrap_or(false)
        || key.is_none()
        || req.uri() != "/socket";
}

#[derive(Debug)]
pub enum ValidationError {
    NoMachineIdSet,
    DeviceNotRegistered,
}

async fn validate_connection(listener_type: &ListenerType, machine_id: &Option<String>, database_pool: Pool) -> Result<(), ValidationError> {
    match listener_type {
        ListenerType::Device => { 
            if machine_id.is_none() { return Err(ValidationError::NoMachineIdSet); };

            let machine_id = machine_id.clone().expect("Expected machine id");

            if !is_device_registered(machine_id, database_pool).await { return Err(ValidationError::DeviceNotRegistered); }
        },
        ListenerType::CLI => {
        },
    } 

    Ok(())
}

fn error_response_json(message: &str, status_code: StatusCode) -> Response<http_body_util::Full<tokio_tungstenite::tungstenite::Bytes>> {
    let body = serde_json::json!({"msg": message, "data": null}).to_string();

    let mut error_response = Response::new(http_body_util::Full::from(body));

    *error_response.status_mut() = status_code;
    error_response.headers_mut().append("content-type", "application/json".parse().unwrap());

    error_response
}

async fn new_session(mut req: Request<Incoming>, addr: SocketAddr, database_pool: Pool) -> Result<Response<Body>, Infallible> {
    info!("New incoming request");

    info!(
        path = req.uri().path(),
        headers = format!("{:#?}", req.headers()),
    );

    let upgrade = HeaderValue::from_static("Upgrade");
    let websocket = HeaderValue::from_static("websocket");
    let headers = req.headers();

    let key = headers.get(SEC_WEBSOCKET_KEY);

    let derived = key.map(|k| derive_accept_key(k.as_bytes()));

    let ver = req.version();

    let machine_id = req.headers()
        .get("machine-id")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| Some(header.to_string()));

    let bearer_token = req.headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| Some(header.to_string()));

    let listener_type = req.headers()
        .get("machine-type")
        .and_then(|header| header.to_str().ok())
        .and_then(|listener_type| {
            match listener_type {
                "device" => {
                    return Some(ListenerType::Device);
                },
                "cli" => {
                    return Some(ListenerType::CLI);
                }
                _ => {
                    return None;
                }
            }
        }).expect("Invalid value for 'machine-type' ({listener_type})") ;

    let validation_result  = validate_connection(
        &listener_type,
        &machine_id,
        database_pool.clone()
    ).await;

    match validation_result {
        Err(ValidationError::DeviceNotRegistered) => {
            return Ok(error_response_json("Device not registered", StatusCode::NOT_FOUND));
        },
        Err(ValidationError::NoMachineIdSet) => {
            return Ok(error_response_json("No machine-id set in header", StatusCode::BAD_REQUEST));
        }
        Ok(()) => {},
    };

    if is_http_connection(&mut req) {
        info!(protocol = "HTTP");
        return Ok(handle_http_connection(&mut req, database_pool).await);
    }

    info!(protocol = "WebSocket");
    debug!("Spawning a new thread...");

    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                let upgraded = TokioIo::new(upgraded);

                handle_connection(ListenerSession {
                        machine_id: machine_id.expect("Error while retrieving header 'machine-id'"),
                        listener_type: listener_type,
                        bearer_token: bearer_token.expect("Error while retrieving header 'authorization'"),
                        ws_stream: WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await,
                }, database_pool.clone())
                .await;
            }
            Err(e) => println!("Failed to upgrade {}", e),
        }
    });

    let mut res = Response::new(Body::default());

    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    *res.version_mut() = ver;

    res.headers_mut().append(CONNECTION, upgrade);
    res.headers_mut().append(UPGRADE, websocket);
    res.headers_mut().append(SEC_WEBSOCKET_ACCEPT, derived.unwrap().parse().unwrap());

    Ok(res)
}

async fn handle_connection(mut session: ListenerSession, database_pool: Pool) {
    match session.listener_type {
        ListenerType::Device => {
            debug!("Listening to device");
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

                        info!(
                            opeartion = "up",
                            environment = environment.name,
                            project = project.name,
                            state = state_operation_message.state,
                        );

                        insert_into(states::dsl::states)
                            .values((
                                states::json.eq(state_operation_message.state.expect("Expected a JSON").clone()),
                                states::environment_id.eq(environment.id),
                            ))
                            .execute(conn).expect("Could not insert state");
                        }).await;
                },
                StateAction::Preview => {
                    unimplemented!("action not implemented yet");
                },
                StateAction::Down => {
                    // Duplicated code
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

                        info!(
                            opeartion = "up",
                            environment = environment.name,
                            project = project.name,
                            state = "",
                        );

                        insert_into(states::dsl::states)
                            .values((
                                states::json.eq("{}".to_string()),
                                states::environment_id.eq(environment.id),
                            ))
                            .execute(conn).expect("Could not insert state");
                        }).await;
                }
            }
        },
        _ => {panic!("Listener type not implemented")}
    }
}

use diesel::{insert_into, prelude::*};
use deadpool_diesel::sqlite::{Manager, Pool, Runtime};
use hyper::{
    body::Incoming, header::{
        HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION,
        UPGRADE,
    }, server::conn::http1, service::service_fn, upgrade::Upgraded, Method, Request, Response, StatusCode, Version
};

use tokio_tungstenite::tungstenite::{
    handshake::derive_accept_key,
    protocol::Role,
};

type Body = http_body_util::Full<hyper::body::Bytes>;

use hyper_util::rt::TokioIo;

#[tokio::main]
async fn main() {
    let format = tracing_subscriber::fmt::format()
        .pretty()
        .with_source_location(false);

    tracing_subscriber::fmt()
        .event_format(format)
        .init();

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
    let port = config.port.unwrap_or(9734u64.into());

    let full_address = format!("{address}:{port}");

    info!("Listening at {full_address}");

    let try_socket = TcpListener::bind(full_address).await;
    let listener = try_socket.expect("Failed to bind");
    
    while let Ok((stream, addr)) = listener.accept().await {
        let pool_ref = pool.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| new_session(req, addr, pool_ref.clone()));

            let io = TokioIo::new(stream);
            let conn = http1::Builder::new().serve_connection(io, service).with_upgrades();

            if let Err(err) = conn.await {
                error!("failed to serve connection: {err:?}");
            }
        });
    }
}
