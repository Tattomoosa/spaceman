// extern crate serde;
use uuid::Uuid;
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectRequest {
    pub msg: String,
    pub version: String,
    pub id: Uuid,
    pub support: Vec<String>
}

impl ConnectRequest {
    pub fn new(uuid: &Uuid) -> Self {
        ConnectRequest {
            msg: "connect".to_string(),
            version: 1.to_string(),
            support: vec!(1.to_string()),
            id: uuid.clone(),
        }
    }
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
