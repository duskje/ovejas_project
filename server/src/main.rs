use std::{net::TcpListener, thread::spawn};

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

use std::fs;

fn main() {
    env_logger::init();

    let server = TcpListener::bind("127.0.0.1:3012").unwrap();

    for stream in server.incoming(){
        // Este move es para que el spawn (que hace uso de hebras)
        // sea dueño todo ese bloque (necesario por garantias de ciclo de vida)
        spawn(move || {
            let callback = |req: &Request, mut response: Response| { // esto es un lambda en Rust
                println!("handshake");
                println!("request path: {}", req.uri().path());

                for (header, _value) in req.headers() {
                    println!("* {header}");
                }

                let headers = response.headers_mut();

                headers.append("authorization", "mi autorizacion".parse().unwrap());

                Ok(response)
            };

            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap(); // recibe la
                                                                                // conexión con
                                                                                // headers, a
                                                                                // diferencia de
                                                                                // accept()
            loop {
                let msg = websocket.read().unwrap(); // bloqueo hasta que se reciba algo

                if msg.is_binary() {
                    if msg.into_data()[0] == 0x10 {
                        println!("message code is 0x10");
                    }

                    let contents = fs::read_to_string("test.json").expect("could not open file");
                    websocket.send(contents.into()).unwrap();
                }
            }
        });
    }
}
