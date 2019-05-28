#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate log;
extern crate simple_logging;
use log::LevelFilter;

use rocket_bot::{
    RocketBot,
    User,
    RocketMessageHandler
};
//use env_logger;


const DOMAIN: &str = "wss://rocket.cat.pdx.edu/websocket";
// const DOMAIN: &str = "echo.websocket.org";
const USERNAME: &str = "rustbot";
const PASSWORD: &str = "zVd/HkU9djMBhloZCwaXf4PveNyP56";

struct HelloHandler {}

impl RocketMessageHandler for HelloHandler {
    fn on_message(self) {
        ();
    }
}


fn main() -> Result<(), ()> {
    simple_logging::log_to_stderr(LevelFilter::Info);
    let _hello_bot = HelloHandler {};
    let user = User::new(
        String::from(USERNAME),
        String::from(PASSWORD)
    );
    /*
    let mut rocket_bot = RocketBot::new(
        String::from(DOMAIN),
        user,
        hello_bot
    );
    match rocket_bot.run() {
        _ => {}
    }
    */
    RocketBot::run(DOMAIN.to_string(), user);

    Ok(())
}
