use std::{fs, net::TcpStream};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

use shared::Operation;

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

fn main() {
    env_logger::init();

    let (mut websocket, response) = connect("ws://localhost:3000/socket").expect("Could not connect to the server");

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
