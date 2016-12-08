extern crate websocket;
extern crate crypto;

use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

fn main() {
    println!("起動中...");
    let server = Server::bind("127.0.0.1:8124").unwrap();
    for connection in server {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let request = connection.unwrap().read_request().unwrap(); // Get the request
            let headers = request.headers.clone(); // Keep the headers so we can check them

            request.validate().unwrap(); // Validate the request

            let mut response = request.accept(); // Form a response

            if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
                if protocols.contains(&("rust-websocket".to_string())) {
                    // We have a protocol we want to use
                    response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
                }
            }

            let mut client = response.send().unwrap(); // Send the response

            let ip = client.get_mut_sender()
                .get_mut()
                .peer_addr()
                .unwrap();

            //println!("Connection from {}", ip);

            let mut sha256 = Sha256::new();
            sha256.input_str(&ip.to_string());
            let client_id = sha256.result_str();

            let message: Message = Message::text(client_id);
            client.send_message(&message).unwrap();

            let (mut sender, mut receiver) = client.split();

            for message in receiver.incoming_messages() {
                let message: Message = message.unwrap();

                match message.opcode {
                    Type::Close => {
                        let message = Message::close();
                        sender.send_message(&message).unwrap();
                        //println!("Client {} disconnected", ip);
                        return;
                    },
                    Type::Pong => {
                        let message = Message::pong(message.payload);
                        //println!("{:?}", message);
                        sender.send_message(&message).unwrap();
                    },
                    Type::Ping => {
                        let message = Message::ping(message.payload);
                        //println!("{:?}", message);
                        sender.send_message(&message).unwrap();
                    },
                    _ => {
                        //println!("{:?}", message);
                        sender.send_message(&message).unwrap();
                    },
                }
            }
        });
    }
}