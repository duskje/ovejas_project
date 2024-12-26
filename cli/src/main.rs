use std::net::TcpStream;

use http::Request;

use ovejas::project::find_project_root;
use ovejas::executor::python_executor;
use tungstenite::{connect, handshake::client::Response, stream::MaybeTlsStream, Message, WebSocket};
use shared::{StateAction, StateOperationMessage};

fn init_conn() -> (WebSocket<MaybeTlsStream<TcpStream>>, Response) {
    let request = Request::builder()
        .uri("ws://localhost:9734/socket")
        .header("sec-websocket-key", "foo")
        .header("machine-type", "cli")
        .header("upgrade", "websocket")
        .header("host", "example.com")
        .header("connection", "upgrade")
        .header("sec-websocket-version", 13)
        .body(())
        .unwrap();

    let (websocket, response) = connect(request).expect("Could not connect to the server");

    println!("Connected successfully to the server!");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");

    for (header, _value) in response.headers() {
        println!("* {header}");
    }

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

    let project_root_dir = find_project_root();
    let target_state = python_executor(project_root_dir.expect("Could not find project root")).unwrap();

    let (mut websocket, response) = init_conn();

    match matches.subcommand() {
        Some(("up", matches)) => {
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Up,
                state: Some(target_state),
            };

            let _ = websocket.send(state_operation.into());

            println!("{response:?}");
        },
        Some(("preview", matches)) => {
            let environment = matches.get_one::<String>("env").expect("Expected environment");

            let state_operation = StateOperationMessage {
                environment: environment.to_string(),
                action: StateAction::Preview,
                state: None,
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
            };
            
            let _ = websocket.send(state_operation.into());

            println!("{response:?}");
        },
        _ => unreachable!("clap should ensure we don't get here"),
    };

    let _ = websocket.send(Message::Close(Option::None));

    Ok(())
}
