extern crate websocket;
extern crate crypto;

use std::thread;
use std::sync::{Mutex, Arc};
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

/// Broadcast.
///
///
///
///
fn broadcast(senders: &mut Vec<websocket::client::Sender<websocket::WebSocketStream>>,
             message: Message) {

    for sender in senders {
        sender.send_message(&message);
    }
}

/// Main.
///
/// ```
///
/// ```
///
///
fn main() {
    println!("起動中...");
    let server: Server = Server::bind("0.0.0.0:8124").unwrap();
    let senders = Arc::new(Mutex::new(Vec::new()));

    for connection in server {
        let senders = senders.clone();
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

            println!("Connection from {}", ip);

            let mut sha256:Sha256 = Sha256::new();
            sha256.input_str(&ip.to_string());
            let client_id = sha256.result_str();

            let message: Message = Message::text(client_id);
            client.send_message(&message).unwrap();

            let (sender, mut receiver) = client.split();
            senders.lock().unwrap().push(sender);

            for message in receiver.incoming_messages() {

                if let Err(e) = message {
                    // コネクションが切れてメッセージ送れない時など
                    println!("Exception: {}", e);


                } else {
                    let message: Message = message.unwrap();


                    match message.opcode {
                        Type::Close => {
                            // クライアント側からコネクション切断された時
                            //senders.lock().unwrap().remove(sender);
                            println!("Client {} disconnected", ip);
                        },
                        Type::Binary => {},
                        Type::Ping => {},
                        Type::Pong => {},
                        Type::Text => {
                            broadcast(&mut *senders.lock().unwrap(), message);
                        }
                    }
                }
            }
        });
    }
}