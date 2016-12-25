/// WebSocket server using trait objects to route
/// to an infinitely extensible number of handlers
extern crate ws;
extern crate env_logger;
extern crate crypto;

use crypto::sha2::Sha256;
use crypto::digest::Digest;

/// Route Struct
struct Router {
    sender: ws::Sender,
    inner: Box<ws::Handler>,
}

/// Routeのイベントハンドラ
impl ws::Handler for Router {
    /// On Request
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
        let out: ws::Sender = self.sender.clone();

        match req.resource() {
            // Routing Config
            "/" => {
                self.inner = Box::new (
                    Echo {
                        ws: out 
                    }
                );
            },

            // Not Found時
            _ => (),
        }

        // Delegate to the child handler
        self.inner.on_request(req)
    }

    /// On ShutDown
    fn on_shutdown(&mut self) {
        self.inner.on_shutdown()
    }

    /// On Open
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        self.inner.on_open(shake)
    }

    /// On Message
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.inner.on_message(msg)
    }

    /// On Close
    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        self.inner.on_close(code, reason)
    }

    /// On Error
    fn on_error(&mut self, err: ws::Error) {
        self.inner.on_error(err)
    }
}

/// This handler returns a 404 response to all handshake requests
struct NotFound;


/// Not Found 時のイベントハンドラ
impl ws::Handler for NotFound {

    /// On Request
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
        let mut res = try!(ws::Response::from_request(req));
        res.set_status(404);
        res.set_reason("Not Found");
        Ok(res)
    }
}


/// This handler simply echoes all messages back to the client
struct Echo {
    ws: ws::Sender,
}

/// Sub Handler
impl ws::Handler for Echo {
    /// On Open
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            let mut sha256: Sha256 = Sha256::new();
            sha256.input_str(&addr.to_string());
            let client_id = sha256.result_str();

            println!("client_ip:{}", addr);
            println!("client_id:{}", client_id);
            let _ = self.ws.send(client_id);
        }
        Ok(())
    }

    /// On Message
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        //println!("client_send_json", msg);
        self.ws.broadcast(msg)
    }

}

/// Main
fn main () {

    println!("起動中...");
    env_logger::init().unwrap();

    if let Err(error) = ws::listen("0.0.0.0:8124", |out: ws::Sender| {
        // 通信できるとき
        Router {
            // Routeの設定にあるURLのとき
            sender: out,
            // Routeの設定にないURLのとき
            inner: Box::new(NotFound),
        }
    }) {
        // 通信失敗時
        println!("Failed to create WebSocket due to {:?}", error);
    }
}