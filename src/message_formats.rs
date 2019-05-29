extern crate crypto;

use uuid::Uuid;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use serde;
// #[macro_use]
// use serde_derive;
use serde::{
    Serialize,
    Deserialize
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestUser {
    pub username: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Password {
    pub digest: String,
    pub algorithm: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub name: String,
    pub password: String
}

// REQUEST FORMATS {{{
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub msg: String,
    pub method: String,
    pub id: Uuid,
    pub params: Vec<RequestParams>
}

impl LoginRequest {
    pub fn new(uuid: Uuid, username: &str, password: &str) -> Self {
        let hashed_password = {
            let mut hasher = Sha256::new();
            hasher.input_str(password);
            hasher.result_str()
        };
        LoginRequest {
            msg: "method".to_string(),
            method: "login".to_string(),
            id: uuid,
            params: vec!(
                RequestParams::User {
                    user: RequestUser {
                        username : username.to_string(),
                    },
                    password: Password {
                        digest : hashed_password,
                        algorithm : "sha-256".to_string()
                    }
                }
            )
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeRequest {
    pub msg: String,
    pub id: Uuid,
    pub name: String,
    pub params: Vec<String>,
}

impl SubscribeRequest {
    pub fn new(name: &str, params: Vec<String>) -> Self {
        SubscribeRequest {
            msg: "sub".to_string(),
            id: Uuid::new_v4(),
            name: name.to_string(),
            params
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectRequest {
    pub msg: String,
    pub version: String,
    // pub id: Uuid,
    pub support: Vec<String>
}

impl ConnectRequest {
    pub fn new() -> Self {
        ConnectRequest {
            msg: "connect".to_string(),
            version: 1.to_string(),
            support: vec!(1.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong {
    pub msg: String
}

impl Pong {
    pub fn new() -> Self {
        Pong {
            msg: "pong".to_string(),
        }
    }
}
// }}}

// RESPONSE FORMATS {{{
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub server_id: Option<String>,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(default)]
    pub error: Option<i32>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    #[serde(rename = "errorType")]
    pub error_type: Option<String>,
    #[serde(default)]
    pub result: Option<Result>,
    // Can't be UUID so making it a string,
    // need to figure out how to make it a uuid later
    // TODO change back to string? not working
    #[serde(default)]
    pub id: Option<ResponseID>
}

// TODO not working??
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseID {
    Uuid(Uuid),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Result {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    #[serde(rename = "tokenExpires")]
    pub token_expires: Option<TokenExpires>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenExpires {
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    type_field: Option<String>
}

// }}}

// USER {{{
#[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type")]
#[serde(untagged)]
pub enum RequestParams {
    User {
        user: RequestUser,
        password: Password,
    },
}

impl User {
    pub fn new(name: String, password: String) -> User {
        User {
            name,
            password
        }
    }
}
// }}}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    user: RequestUser,
    password: Password,
}

// vim: foldmethod=marker
