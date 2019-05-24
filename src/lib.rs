#![allow(dead_code)]

extern crate ws;
extern crate serde;
extern crate uuid;
extern crate crypto;
#[macro_use] extern crate log;
// extern crate env_logger;
// extern crate serde_json;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
//use simple_logging::SimpleLogger;
// use crypto::sha2::Sha256;
use uuid::Uuid;

use serde::{
    Serialize,
    Deserialize
};
/*
use serde_json::{

};
*/

pub trait MessageHandler {
    fn on_message_recieved(self) -> Result<i32, &'static str>;
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestUser {
    username: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Password {
    digest: String,
    algorithm: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    name: String,
    password: String
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginRequest {
    msg: String,
    method: String,
    id: Uuid,
    params: Vec<RequestParams>
}

#[derive(Serialize, Deserialize, Debug)]
struct ConnectRequest {
    msg: String,
    version: String,
    support: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type")]
#[serde(untagged)]
enum RequestParams {
    User {
        user: RequestUser,
        password: Password,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct UserInfo {
    user: RequestUser,
    password: Password,
}

impl User {
    pub fn new(name: String, password: String) -> User {
        User {
            name,
            password
        }
    }
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

    pub fn run(&mut self) {
        info!("Connecting...");
        match self.connect() {
            Ok(()) => {
                println!("CONNECTION SUCCESS!");
            },
            Err(ws_err) => {
                println!("ERROR");
                println!("{:?}", ws_err);
            }
        }
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
    }

    fn connect(&mut self) -> Result<(), ws::Error> {
        let domain = format!("ws://{}", self.domain);
        let request = ConnectRequest {
            msg: "connect".to_string(),
            version: "1".to_string(),
            support: vec!(
                "1".to_string(),
                "pre2".to_string(),
                "pre1".to_string())
        };

        ws::connect(domain, |out| {
            out.send(serde_json::to_string::<ConnectRequest>(&request).unwrap()).unwrap();
            move |message: ws::Message| {
                println!("Got response: {:?}", message);
                // out.close(ws::CloseCode::Normal)
                Ok(())
            }
        })
    }

    fn login(&mut self) -> Result<(), ws::Error> {
        let domain = format!("ws://{}", self.domain);
        println!("Username: {}", self.user.name);
        println!("Password: {}", self.user.password); // TODO hash?
        println!("URL: {}", domain);
        let uuid = Uuid::new_v4();
        let mut password = self.user.password.clone();
        let hashed_password = {
            let mut hasher = Sha256::new();
            hasher.input_str(password.as_mut_str());
            hasher.result_str()
        };

        let request = LoginRequest {
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
                        // digest : RocketBot::<H>::get_hash(self.user.password.as_mut_str().clone()),
                        algorithm : "sha-256".to_string()
                    }
                }
            )
        };
        ws::connect(domain, |out| {
            out.send(serde_json::to_string::<LoginRequest>(&request).unwrap()).unwrap();
            move |message: ws::Message| {
                println!("Got response: {:?}", message);
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

/*
       def _login(self):
        self.logger.info("Logging in as {}\n".format(self.user))
        login_request = {
            "msg": "method",
            "method": "login",
            "id": self.login_id,
            "params": [
                {
                    "user": {"username": self.user},
                    "password": {
                        "digest": self.passhash,
                        "algorithm":"sha-256"
                    }
                }
            ]
        }
        self.ws.send(json.dumps(login_request))

   */

/*
{
    "bot": false,
    "channel_id": "WrCqE6waNtPringLes",
    "channel_name": "#robots",
    "isEdited": false,
    "message_id": "Ytz6gJtjL58JpXsAz",
    "text": "He, I'm saying something here",
    "timestamp": 1539136413276,
    "token": null,
    "user_id": "RDwsdsdfSDSDFdf",
    "user_name": "somecat",
    "_rawMessage" {}
}

   */
