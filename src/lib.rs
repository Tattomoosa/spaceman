#![allow(dead_code)]
#![allow(unused_imports)]

extern crate http;
extern crate crypto;
use crypto::sha2::Sha256;
extern crate ws;
extern crate reqwest;
// extern crate serde;
extern crate uuid;
extern crate openssl;
#[macro_use] extern crate log;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;

use uuid::Uuid;
use crypto::digest::Digest;
use ws::{
    Sender,
    Message,
    Handshake,
    Handler,
    Error,
    CloseCode,
};

use std::collections::HashMap;
use reqwest::{
    header::{
        HeaderName,
        HeaderValue
    }
};

mod message_formats;
use message_formats::*;
pub use message_formats::User;

const X_AUTH_TOKEN: &str = "X-Auth-Token";
const X_USER_ID: &str = "X-User-Id";

pub trait RocketMessageHandler {
    fn on_message(&mut self, msg: String);
}

pub struct DefaultRocketHandler {}

impl RocketMessageHandler for DefaultRocketHandler {
    fn on_message(&mut self, msg: String) {
        info!("MESSAGE: {}", msg);
    }
}

pub struct RocketBot<T>
where T: RocketMessageHandler {
    out: Sender,
    domain: String,
    login_id: Uuid,
    user: User,
    rest_client: reqwest::Client,
    handler: T,
    user_token: Option<String>,
    user_id: Option<String>,
    is_logged_in: bool,
}

impl<T> RocketBot<T>
where T: RocketMessageHandler {

    pub fn start<F>(
        domain: &str,
        user: User,
        handler_factory: F
    )
    where F: Fn() -> T
    {
        let login_id = Uuid::new_v4();
        let wss_url = format!("wss://{}/websocket", domain);
        ws::connect(wss_url, |out| {
            let handler = handler_factory();
            RocketBot {
                out,
                login_id,
                domain: String::from(domain),
                user: user.clone(),
                rest_client: reqwest::Client::new(),
                handler: handler,
                user_token: None,
                user_id: None,
                is_logged_in: false,
            }
        }).unwrap();
    }

    fn connect(&mut self) {
        info!("Connecting");
        // TODO should this use same uuid as login??
        let connect_request = ConnectRequest::new();
        let connect_str = serde_json::to_string::<ConnectRequest>(
            &connect_request).unwrap();
        let _connect_sent = self.out.send(connect_str);
    }

    fn login(&mut self) -> ws::Result<()> {
        info!("Logging in");
        // TODO should thi suse the same uuid as connect?
        let login_request = LoginRequest::new(self.login_id,
                                              self.user.name.as_str(),
                                              self.user.password.as_str());
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

    fn handle_result(&mut self, response: Response) -> ws::Result<()> {
        // TODO deal with no result result response? Can that happen?
        // login!
        info!("HANDLING RESULT..");
        let result = match response.result {
            Some(result) => result,
            None => {
                error!("Result recieved with no result field");
                return Ok(());
            },
        };
        match result {
            RcResult::List(_result) => {
                info!("RESULT IS A LIST");
            },
            RcResult::LoginResult(result) => {
                info!("RESULT IS A LOGIN RESULT");
                match response.id.clone() {
                    Some(id) => {
                        match id {
                            ResponseID::Uuid(id) => {
                                info!("ID IS UUID");
                                if id == self.login_id {
                                    info!("Logged in!");
                                    self.user_id = result.id;
                                    self.user_token = result.token;
                                    self.is_logged_in = true;
                                    self.on_login();
                                }
                            }
                            ResponseID::String(_string) => info!("ID IS String"),
                        }
                    }
                    None => {
                        info!("NO ID");
                    }
                }
            }
        }
        self.handler.on_message(String::from("HELLO"));
        Ok(())
    }

    fn on_login(&mut self) {
        self.subscribe_to_self_events();
        self.get_subscriptions();
    }

    fn get_subscriptions(&mut self) {
        let request = Request {
            msg: String::from("method"),
            method: String::from("subscriptions/get"),
            id: self.user_id.as_ref().unwrap().clone(),
            params: vec![],
        };
        let request_string = serde_json::to_string(&request).unwrap();
        let _subscribed = self.send(request_string);
    }

    // Not sure this does what I expected it to.
    fn subscribe_to_self_events(&mut self) {
        if !self.is_logged_in {
            return;
        }
        info!("Subscribing to events...");
        let event = Parameter::STRING(String::from("event"));
        let b = Parameter::BOOL(false);
        let params = vec!(event, b);
        let subscribe_request = SubscribeRequest::new("stream-notify-all", params);
        let subscribe_string = serde_json::to_string::<SubscribeRequest>(&subscribe_request).unwrap();
        let _subscribed = self.send(subscribe_string);
    }
}

impl<T> Handler for RocketBot<T>
where T: RocketMessageHandler {

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        info!("Connection open");
        self.connect();
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        let msg_txt = msg.into_text().unwrap();
        info!("unwrapping {}", msg_txt);
        let response: serde_json::Result<Response> = serde_json::from_str(&msg_txt);
        match response {
            Ok(response) => {
                let message = match &response.msg {
                    Some(m) => m.as_str(),
                    None => "",
                };
                // Main 'switch'
                match message {
                    "connected" => { self.login()?; },
                    "ping" => { self.pong()?; },
                    "result" => {
                        info!("found result: '{}'", &msg_txt);
                        self.handle_result(response)?;
                    },
                    _ => { error!("UNHANDLED MESSAGE: '{}'", &msg_txt); },
                }
            },
            Err(error) => error!("PARSE ERROR {}", error)
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
