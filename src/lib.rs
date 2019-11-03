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
    fn on_message(self);
}

pub struct DefaultRocketHandler {}

impl RocketMessageHandler for DefaultRocketHandler {
    fn on_message(self) {
        info!("here");
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
        let _connect_sent = self.out.send(
            serde_json::to_string::<ConnectRequest>(&connect_request).unwrap());
        // TODO deal with this, it always returns ok anyway rn
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

    // TODO deal with errors
    fn get_rest(&mut self, method: &str) -> ws::Result<()> {
        let api_endpoint = format!("https://{}{}", self.domain, method);
        let response = self.rest_client.get(api_endpoint.as_str())
            // TODO HEADERS AGH
            /*
            .header(
                HeaderName::from_static(X_AUTH_TOKEN),
                http::header(
                    &self.user_token.unwrap().as_str().clone()))
            .header(
                HeaderName::from_static(X_USER_ID),
                http::header(
                    &self.user_id.unwrap().as_str().clone()))
                // http::convert::HttpTryFrom(&self.user_id.unwrap())
                // &self.user_id.unwrap().clone()
                // )
                */
            .send();
        info!("Sending rest request to: {}", api_endpoint);
        match response {
            Ok(r) => info!("REST response is:\n{:?}", r),
            Err(r) => error!("ERROR:\n{:?}", r)
        };
        Ok(())
    }

    fn handle_result(&mut self, response: Response) -> ws::Result<()> {
        // TODO deal with no result result response? Can that happen?
        // login!
        let result = match response.result {
            Some(result) => result,
            None => {
                error!("Result recieved with no result field");
                return Ok(());
            },
        };
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
        // self.handler.on_message();
        Ok(())
    }

    fn on_login(&mut self) {
        self.subscribe_to_self_events();
    }

    fn subscribe_to_self_events(&mut self) {
        if !self.is_logged_in {
            return;
        }
        // let id = Uuid::new_v4();
        info!("Subscribing to self events...");
        // TODO what is self._user_event_key in the python???
        // let user_event_key = format!("{}/rooms-changed", self.user_id.clone().unwrap());
        // let user_event_key = format!("{}/rooms-changed", self.user_id.clone().unwrap());
        // TODO OK so somehow i need params to be an array with both a string
        // (user_event_key) and a bool... hm
        // let params = vec!(user_event_key, "false".to_string());
        // TODO gets an error message i can't unwrap...
        // let params = vec!("event".to_string(), "false".to_string());
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
        let response : Response = serde_json::from_str(&msg_txt).unwrap();
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
