use tungstenite::{connect, Message};

fn main() {
    env_logger::init();

    let (mut socket, response) = connect("ws://localhost:3012/socket").expect("Can't connect");

    println!("connected");
    println!("http status code: {}", response.status());
    println!("response headers:");

    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    socket.send(Message::Binary(vec![0x10].into())).unwrap();
    
    loop {
        let msg = socket.read().expect("Error reading message");
        println!("received: {msg}");
    }
}
