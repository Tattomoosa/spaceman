#![allow(dead_code)]

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

mod message_formats;
use message_formats::*;
pub use message_formats::User;
// use serde_json;

pub trait MessageHandler {
    fn on_message_recieved(self) -> Result<i32, &'static str>;
}

pub struct RocketBot<H: MessageHandler> {
    domain: String,
    user: User,
    logged_in: bool,
    message_handler: H,
}

impl<H: MessageHandler> RocketBot<H> {
    pub fn new(
        domain: String,
        user: User,
        message_handler: H
        ) -> RocketBot<H> {
        RocketBot {
            domain,
            user,
            logged_in: false,
            message_handler,
        }
    }

    pub fn run(&mut self) -> Result<(), ws::Error> {
        self.connect()
    }

    fn connect(&mut self) -> Result<(), ws::Error> {
        let domain = format!("wss://{}", self.domain);
        let uuid = Uuid::new_v4();
        let connect_request = ConnectRequest::new(&uuid);
        let mut password = self.user.password.clone();
        let hashed_password = {
            let mut hasher = Sha256::new();
            hasher.input_str(password.as_mut_str());
            hasher.result_str()
        };

        let login_request = LoginRequest {
            msg: "method".to_string(),
            method: "login".to_string(),
            id: uuid,
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

        ws::connect(domain, |out| {
            let connect_sent = out.send(
                serde_json::to_string::<ConnectRequest>(&connect_request).unwrap());
            match connect_sent.is_err() {
                false => info!("Client sent connect message."),
                true => error!("CONNECT IS ERROR"),
            }
            let login_sent = out.send(serde_json::to_string::<LoginRequest>(&login_request).unwrap());
            match login_sent.is_err() {
                false=> info!("Client sent login message."),
                true => error!("LOGIN IS ERROR"),
            }
            move |msg| {
                info!("Incoming message: {}", msg);
                out.close(ws::CloseCode::Normal)
            }
        })
    }

    fn get_hash(string: &mut str) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(string);
        return hasher.result_str();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
