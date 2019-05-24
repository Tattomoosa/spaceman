use uuid::Uuid;
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

#[derive(Serialize, Deserialize, Debug)]
struct ResponseFormat<'a> {
    pub msg: Option<&'a str>,
    pub id: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorResponse<'a> {
    pub error: i32,
    pub reason: Option<&'a str>,
    pub message: Option<&'a str>,
    #[serde(rename = "errorType")]
    pub error_type: Option<&'a str>
}

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
