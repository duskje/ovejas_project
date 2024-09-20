use std::{fs, net::TcpStream};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

enum OpCode {
    RequestState,
    ExecuteTransaction,
}

impl From<Message> for OpCode {
    fn from(orig: Message) -> Self {
        let op_code = orig.into_data()[0];

        match op_code {
            0x10 => return OpCode::RequestState,
            _ => panic!("Unknown opcode: {}", op_code),
        }
    }
}

fn listen(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>){
    let msg = socket.read().expect("Error reading message");

    if !msg.is_binary() {
        panic!("Invalid read!");
    }

    let op_code: OpCode = msg.into();

    match op_code {
        OpCode::RequestState => {
            println!("Server requested current state.");

            let contents = fs::read_to_string("local_state.json").expect("could not open file");
            socket.send(Message::Binary(contents.into())).expect("Could not send state to remote");
        },
        OpCode::ExecuteTransaction => {},
    }
}

fn main() {
    env_logger::init();

    let (mut websocket, response) = connect("ws://localhost:3012/socket").expect("Could not connect to the server");

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
