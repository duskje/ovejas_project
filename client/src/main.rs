use std::fs;
use tungstenite::{connect, Message};

fn main() {
    env_logger::init();

    let (mut socket, response) = connect("ws://localhost:3012/socket").expect("Could not connect to the server");

    println!("Connected successfully to the server!");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");

    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    loop {
        let msg = socket.read().expect("Error reading message");

        if !msg.is_binary() {
            panic!("Invalid read!");
        }

        if msg.into_data()[0] == 0x10 {
            println!("Server requested current state.");

            let contents = fs::read_to_string("local_state.json").expect("could not open file");
            socket.send(Message::Binary(contents.into())).expect("Could not send state to remote");
        }
    }
}
