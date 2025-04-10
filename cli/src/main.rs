use std::net::TcpStream;
use std::fs::File;
use std::io::prelude::*;

use clap::{Arg, ArgAction};
use figment::providers::{Format, Env, Yaml};
use figment::Figment;
use http::{Request, StatusCode};

use pyo3::ffi::PyErr_SetInterrupt;
use serde::{Deserialize, Serialize};
use tungstenite::{connect, handshake::client::Response, stream::MaybeTlsStream, Message, WebSocket};
use toml::{self, Value};

use ovejas::project::find_project_root;
use ovejas::executor::python_executor;
use ovejas::rest::{UserCreateDTO, UserDeleteDTO, DeviceDeleteDTO, DeviceCreateDTO, EnrollDeviceDTO};
use shared::state_operations::{StateAction, StateOperationMessage};
use tungstenite::error::Error;

use uuid::Uuid;

fn init_conn(full_addr: String, cli_token: String) -> (WebSocket<MaybeTlsStream<TcpStream>>, Response) {
    let request = Request::builder()
        .uri(format!("ws://{full_addr}/socket"))
        .header("sec-websocket-key", "foo")
        .header("machine-type", "cli")
        .header("machine-id", "cli")
        .header("upgrade", "websocket")
        .header("host", "example.com")
        .header("connection", "upgrade")
        .header("authorization", cli_token)
        .header("sec-websocket-version", 13)
        .body(())
        .unwrap();

    let (websocket, response) = connect(request).inspect_err(|error| {
        match error {
            Error::Http(response) => {
                match response.status() {
                    StatusCode::FORBIDDEN  => panic!("Authentication error"),
                    _ => unimplemented!("Status code not covered"),
                }

            }
            _ => println!("Could not connect to the server")
        }
    }).expect("Could not connect to the server");

    println!("Connected successfully to remote");

    return (websocket, response);
}

struct ProjectMetadata {
    project_name: String,
}

#[derive(Debug)]
enum ProjectError {
    NotFoundError(String),
    PythonExecutorError(String),
    FailedToReadError,
}

fn get_project_metadata() -> Result<ProjectMetadata, ProjectError> {
    let project_root_dir = find_project_root()
        .ok_or(ProjectError::NotFoundError(String::from("Could not find project root.")))?;

    let mut pyproject_file = File::open(format!("{project_root_dir}/pyproject.toml"))
        .expect("Could not find pyproject.toml in project");

    let mut contents = String::new();

    pyproject_file.read_to_string(&mut contents).expect("Failed to read pyproject.toml");

    let project_name = toml::from_str::<Value>(contents.as_str())
        .expect("Failed to deserialize pyproject.toml")
        .get("tool")
        .unwrap()
        .get("poetry")
        .unwrap()
        .get("name")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    Ok(ProjectMetadata {
        project_name,
    })
}

fn get_target_state() -> Result<String, ProjectError> {
    let project_root_dir = find_project_root()
        .ok_or(ProjectError::NotFoundError(String::from("Could not find project root.")))?;

    let target_state = python_executor(project_root_dir.clone())
        .map_err( |err| ProjectError::PythonExecutorError(err.to_string() ))?;

    Ok(target_state)
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerResponse {
    msg: String,
    data: serde_json::Value,
}

#[derive(Deserialize)]
struct Config {
    port: Option<u64>,
    address: Option<String>,
    cli_token: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = clap::Command::new("ovejas")
        .bin_name("ovejas")
        .subcommand_required(true)
        .subcommand(
            clap::command!("up").arg(
                clap::arg!(-e --env <ENVIRONMENT>)
                .required(true)
                .value_parser(clap::value_parser!(String)),
            )
        )
        .subcommand(
            clap::command!("preview").arg(
                clap::arg!(-e --env <ENVIRONMENT>)
                .required(true)
                .value_parser(clap::value_parser!(String)),
            )
        )
        .subcommand(
            clap::command!("down").arg(
                clap::arg!(-e --env <ENVIRONMENT>)
                .required(true)
                .value_parser(clap::value_parser!(String)),
            )
        )
        .subcommand(
            clap::command!("device")
            .subcommand(
                clap::command!("write")
                    .arg(
                        clap::arg!(-n --name <NAME>)
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                    )
                    .arg(
                        Arg::new("machine-id")  
                        .short('i')
                        .long("machine-id")
                        .action(ArgAction::Set)
                        .value_name("UUID")
                    )
            )
            .subcommand(
                clap::command!("delete").arg(
                    clap::arg!(-n --name <NAME>)
                    .required(true)
                    .value_parser(clap::value_parser!(String)),
                )
            )
        )
        .subcommand(
            clap::command!("environment")
                .arg(clap::arg!(-e --env <ENV>))
                .subcommand(
                    clap::command!("add-user").arg(
                        clap::arg!(-n --name <NAME>)
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                    )
                )
                .subcommand(
                    clap::command!("del-user").arg(
                            clap::arg!(-n --name <NAME>)
                            .required(true)
                            .value_parser(clap::value_parser!(String)),
                    )
                )
                .subcommand(
                    clap::command!("add-device").arg(
                        Arg::new("machine-id")  
                        .short('i')
                        .long("machine-id")
                        .action(ArgAction::Set)
                        .value_name("UUID")
                    )
                )
                .subcommand(
                    clap::command!("del-device").arg(
                            clap::arg!(-n --name <NAME>)
                            .required(true)
                            .value_parser(clap::value_parser!(String)),
                    )
                )
        )
        .subcommand(
            clap::command!("user")
            .subcommand(
                clap::command!("write").arg(
                    clap::arg!(-n --name <NAME>)
                    .required(true)
                    .value_parser(clap::value_parser!(String)),
                )
            )
            .subcommand(
                clap::command!("delete").arg(
                    clap::arg!(-n --name <NAME>)
                    .required(true)
                    .value_parser(clap::value_parser!(String)),
                )
            )
        );

    let matches = cmd.get_matches();

    let config: Config = Figment::new()
        .merge(Yaml::file("config.yml"))
        .join(Env::raw().only(&["PORT", "ADDRESS", "CLI_TOKEN"]))
        .extract().unwrap();

    let address = config.address.unwrap_or("127.0.0.1".into());
    let port = config.port.unwrap_or(9734u64.into());
    let cli_token = config.cli_token.expect("no cli_token");

    let full_addr = format!("{address}:{port}");

    match matches.subcommand() {
        Some(("up", matches)) => {
            let (mut websocket, response) = init_conn(full_addr, cli_token);
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let project_metadata = get_project_metadata().unwrap();
            let target_state = get_target_state().unwrap();

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Up,
                state: Some(target_state),
                project: project_metadata.project_name,
            };

            println!("target state pushed to remote");

            let _ = websocket.send(state_operation.into());

            websocket.send(Message::Close(Option::None)).unwrap();
        },
        Some(("preview", matches)) => {
            let (mut websocket, response) = init_conn(full_addr, cli_token);
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let project_metadata = get_project_metadata().unwrap();
            let target_state = get_target_state().unwrap();

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Preview,
                state: None,
                project: project_metadata.project_name,
            };
            
            let _ = websocket.send(state_operation.into());

            println!("{}", target_state);

            websocket.send(Message::Close(Option::None)).unwrap();
        },
        Some(("down", matches)) => {
            let (mut websocket, response) = init_conn(full_addr, cli_token);
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let project_metadata = get_project_metadata().unwrap();
            let target_state = get_target_state().unwrap();

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Down,
                state: Some(target_state),
                project: project_metadata.project_name,
            };
            
            let _ = websocket.send(state_operation.into());

            println!("{response:?}");

            websocket.send(Message::Close(Option::None)).unwrap();
        },
        Some(("device", matches)) => {
            match matches.subcommand() {
                Some(("write", matches)) => {
                    let name = matches.get_one::<String>("name").expect("Expected name");
                    let machine_id = matches.get_one::<String>("machine-id").expect("Expected machine-id");

                    let device_create_dto = DeviceCreateDTO {
                        name: name.to_string(),
                        machine_id: machine_id.to_string(),
                    };

                    println!("{device_create_dto:?}");

                    let client = reqwest::blocking::Client::new();

                    let response = client.post(format!("http://{full_addr}/device"))
                        .json(&device_create_dto)
                        .header("machine-type", "cli")
                        .header("Authorization", cli_token).send().unwrap();

                    println!("{:?}", response.json::<ServerResponse>());
                },
                Some(("delete", matches)) => {
                    let name = matches.get_one::<String>("name").expect("Expected name");

                    let device_delete_dto = DeviceDeleteDTO {
                        name: name.to_string(),
                    };

                    let client = reqwest::blocking::Client::new();

                    let response = client.delete(format!("http://{full_addr}/device"))
                        .json(&device_delete_dto)
                        .header("machine-type", "cli")
                        .header("Authorization", cli_token).send().unwrap();

                    println!("{:?}", response.json::<ServerResponse>());

                },
                _ => unreachable!("Clap should ensure we don't get here"),
            }
        },
        Some(("environment", matches)) => {
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            match matches.subcommand() {
                Some(("add-device", matches)) => {
                    let machine_id = matches.get_one::<String>("machine-id").expect("Expected machine-id");

                    let project_metadata = get_project_metadata().unwrap();

                    let device_create_dto = EnrollDeviceDTO {
                        machine_id: machine_id.to_string(),
                        environment_name: environment.to_string(),
                        project_name: project_metadata.project_name,
                    };

                    println!("{device_create_dto:?}");

                    let client = reqwest::blocking::Client::new();

                    let response = client.post(format!("http://{full_addr}/enroll_device"))
                        .json(&device_create_dto)
                        .header("machine-type", "cli")
                        .header("Authorization", cli_token).send().unwrap();

                    println!("{:?}", response.json::<ServerResponse>());
                },
                _ => unreachable!("Clap should ensure we don't get here"),
            }
        },
        Some(("user", matches)) => {
            match matches.subcommand() {
                Some(("write", matches)) => {
                    let name = matches.get_one::<String>("name").expect("Expected name");

                    let device_create_dto = UserCreateDTO {
                        name: name.to_string(),
                        password: String::from("placeholder-password"),
                    };

                    println!("{device_create_dto:?}");

                    let client = reqwest::blocking::Client::new();

                    let response = client.post(format!("http://{full_addr}/user"))
                        .json(&device_create_dto)
                        .header("machine-type", "cli")
                        .header("Authorization", cli_token).send().unwrap();

                    println!("{:?}", response.json::<ServerResponse>());
                },
                Some(("delete", matches)) => {
                    let name = matches.get_one::<String>("name").expect("Expected name");

                    let device_delete_dto = DeviceDeleteDTO {
                        name: name.to_string(),
                    };

                    let client = reqwest::blocking::Client::new();

                    let response = client.delete(format!("http://{full_addr}/user"))
                        .json(&device_delete_dto)
                        .header("machine-type", "cli")
                        .header("Authorization", cli_token).send().unwrap();

                    println!("{:?}", response.json::<ServerResponse>());
                
                },
                _ => unreachable!("Clap should ensure we don't get here"),
            }
        },
        _ => unreachable!("Clap should ensure we don't get here"),
    };

    Ok(())
}
