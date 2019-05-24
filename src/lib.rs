#![allow(dead_code)]

extern crate ws;
extern crate serde;
extern crate uuid;
extern crate crypto;
extern crate openssl;
#[macro_use] extern crate log;

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
        /*
        let domain = format!("ws://{}/websocket", self.domain);
        info!("Connecting to domain: {}", domain);
        let connect_request = ConnectRequest::new();

        ws::connect(domain, |out| {
            let connect = out.send(
                serde_json::to_string::<ConnectRequest>(&connect_request).unwrap()
                );
            if !connect.is_err() {
                info!("Sent connection request.");
            }
            else {
                error!("Failed to send connection request.");
            }
            let login = out.send(
                serde_json::to_string::<ConnectRequest>(&connect_request).unwrap()
                );
            move |msg| {
                info!("Client recieved message: {}", msg);
                out.close(ws::CloseCode::Normal)
            }
        })
        */
        self.connect()

            /*
        match self.connect() {
            Ok(()) => {
            },
            Err(ws_err) => {
                println!("ERROR");
                println!("{:?}", ws_err);
            }
            _ => {}
        }
            */
        /*
        println!("Logging in...");
        match self.login() {
            Ok(()) => {
                println!("LOGIN SUCCESS!");
            },
            Err(ws_err) => {
                println!("ERROR");
                println!("{:?}", ws_err);
            }
        }
        */
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
            move |msg| {
                info!("Incoming message: {}", msg);
                out.close(ws::CloseCode::Normal)
            }
            /*
            let login_sent = out.send(serde_json::to_string::<LoginRequest>(&login_request).unwrap());
            match login_sent.is_err() {
                false=> info!("Client sent login message."),
                true => error!("LOGIN IS ERROR"),
            }
            */
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
