/*
In order to run this example, you must first create a .env file in the
root project folder (with Cargo.toml etc), something like:
BOTNAME="mybot"
PASSWORD="password"
DOMAIN="rocket.my.domain.com"
*/

#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate log;
extern crate simple_logging;
use log::LevelFilter;
use dotenv;

use rocket_bot::{
    RocketBot,
    User,
    RocketMessageHandler
};

struct HelloHandler {}
impl RocketMessageHandler for HelloHandler {
    fn on_message(&mut self) {
        println!("CUSTOM HANDLING!")
    }
}


fn main() -> Result<(), ()> {
    simple_logging::log_to_stderr(LevelFilter::Info);
    let username = dotenv::var("BOTNAME").unwrap();
    let password = dotenv::var("PASSWORD").unwrap();
    let domain = dotenv::var("DOMAIN").unwrap();
    println!("{}, {}, {}", username, password, domain);
    let user = User::new(username, password);
    RocketBot::start(&domain, user, move || { HelloHandler {} });
    Ok(())
}
