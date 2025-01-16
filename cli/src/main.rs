use std::net::TcpStream;
use std::fs::File;
use std::io::prelude::*;

use http::{Request, StatusCode};

use tungstenite::{connect, handshake::client::Response, stream::MaybeTlsStream, Message, WebSocket};
use toml::{self, Value};

use ovejas::project::find_project_root;
use ovejas::executor::python_executor;
use shared::state_operations::{StateAction, StateOperationMessage};
use tungstenite::error::Error;

fn init_conn() -> (WebSocket<MaybeTlsStream<TcpStream>>, Response) {
    let request = Request::builder()
        .uri("ws://localhost:9734/socket")
        .header("sec-websocket-key", "foo")
        .header("machine-type", "cli")
        .header("machine-id", "cli-test")
        .header("upgrade", "websocket")
        .header("host", "example.com")
        .header("connection", "upgrade")
        .header("authorization", "Bearer")
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

    // let (websocket, response) = connect(request).expect("Could not connect to the server");

//     println!("Connected successfully to the server!");
//     println!("HTTP status code: {}", response.status());
//     println!("Response headers:");
// 
//     for (header, _value) in response.headers() {
//         println!("* {header}");
//     }
//
//    match response.status() {
//        StatusCode::SWITCHING_PROTOCOLS => 
//        StatusCode::FORBIDDEN  => panic!("Authentication error"),
//        _ => unimplemented!("Status code not covered"),
//    }

    return (websocket, response);
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
        );

    let matches = cmd.get_matches();

    let project_root_dir = find_project_root().expect("Could not find project root");
    let target_state = python_executor(project_root_dir.clone()).expect("Could not open project, make sure you're in the right venv'");
    let mut pyproject_file = File::open(format!("{project_root_dir}/pyproject.toml")).expect("Could not find pyproject.toml in project");
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

    let (mut websocket, response) = init_conn();

    match matches.subcommand() {
        Some(("up", matches)) => {
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Up,
                state: Some(target_state),
                project: project_name,
            };

            let _ = websocket.send(state_operation.into());
        },
        Some(("preview", matches)) => {
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Preview,
                state: None,
                project: project_name,
            };
            
            let _ = websocket.send(state_operation.into());

            println!("{target_state}")
        },
        Some(("down", matches)) => {
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Down,
                state: Some(target_state),
                project: project_name,
            };
            
            let _ = websocket.send(state_operation.into());

            println!("{response:?}");
        },
        _ => unreachable!("Clap should ensure we don't get here"),
    };

    let _ = websocket.send(Message::Close(Option::None));

    Ok(())
}
