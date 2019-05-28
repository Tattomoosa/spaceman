#![allow(dead_code)]
#![allow(unused_imports)]

extern crate ws;
// extern crate serde;
extern crate uuid;
extern crate crypto;
extern crate openssl;
#[macro_use] extern crate log;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;

use uuid::Uuid;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use ws::{
    Sender,
    Message,
    Handshake,
    Handler,
    Error,
    CloseCode,
};

mod message_formats;
use message_formats::*;
pub use message_formats::User;
// use serde_json;

pub trait RocketMessageHandler {
    fn on_message(self);
}

pub struct DefaultRocketHandler {}

impl RocketMessageHandler for DefaultRocketHandler {
    fn on_message(self) {
        info!("here");
    }
}

pub struct RocketBot {
    out: Sender,
    uuid: Uuid,
    user: User,
    handler: Box<RocketMessageHandler>,
}

impl RocketBot {

    pub fn run(
        domain: String,
        user: User
        ) {
        let uuid = Uuid::new_v4();
        ws::connect(domain, |out| {
            RocketBot {
                out,
                uuid,
                user: user.clone(),
                handler: Box::new(DefaultRocketHandler{})
            }
        }).unwrap();
    }

    fn connect(&mut self) {
        info!("Connecting");
        let connect_request = ConnectRequest::new(&self.uuid);
        let _connect_sent = self.out.send(
            serde_json::to_string::<ConnectRequest>(&connect_request).unwrap());
    }

    fn login(&mut self) -> ws::Result<()> {
        info!("Logging in");
        let hashed_password = {
            let mut hasher = Sha256::new();
            let mut password = self.user.password.clone();
            hasher.input_str(password.as_mut_str());
            hasher.result_str()
        };

        let login_request = LoginRequest {
            msg: "method".to_string(),
            method: "login".to_string(),
            id: self.uuid,
            params: vec!(
                RequestParams::User {
                    user: RequestUser {
                        username : self.user.name.clone(),
                    },
                    password: Password {
                        digest : hashed_password,
                        algorithm : "sha-256".to_string()
                    }
                }
            )
        };

        self.send(serde_json::to_string::<LoginRequest>(&login_request).unwrap())
    }

    fn pong(&mut self) -> ws::Result<()> {
        let message = serde_json::to_string::<Pong>(&Pong::new()).unwrap();
        self.send(message)
    }

    fn send(&mut self, message: String) -> ws::Result<()> {
        info!("Sending: {}", message);
        self.out.send(message)
    }
}

impl Handler for RocketBot {

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        info!("Connection open");
        self.connect();
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        let msg2 = msg.into_text().unwrap();
        let msg_obj : ResponseFormat = serde_json::from_str(&msg2).unwrap();
        match msg_obj.msg {
            Some(m) => {
                match m.as_str() {
                    "connected" => { self.login()?; },
                    "ping" => { self.pong()?; },
                    _ => { info!("UNHANDLED MESSAGE: '{}'", &msg2); },
                }
            }
            None => {}
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("WebSocket closing for ({:?}) {}", code, reason);
        info!("Shutting down");
        self.out.shutdown().unwrap();
    }

    fn on_error(&mut self, err: Error) {
        error!("Shutting down server due to error: {}", err);
        self.out.shutdown().unwrap();
    }

}

// TODO make tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
