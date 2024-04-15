#![warn(clippy::pedantic, clippy::perf, missing_docs, clippy::missing_docs_in_private_items)]

#![doc = include_str!("../README.md")]

mod networking;

use std::{error::Error, io, process::ExitCode};
use std::collections::HashMap;
use std::time::Duration;
use oxine::server::Config;
use crate::networking::IdleServer;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> ExitCode {
    simplelog::TermLogger::init(
        if cfg!(debug_assertions) {
            simplelog::LevelFilter::Trace
        } else {
            simplelog::LevelFilter::Info
        },
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto
    ).expect("no logger has been initialized yet");
    
    let res: Result<(), Box<dyn Error>> = inner_main().await.map_err(Into::into);
    let Err(err) = res else { return ExitCode::SUCCESS; };
    error!("~~~ ENCOUNTERED FATAL ERROR ~~~");
    error!("{err}");
    ExitCode::FAILURE
}

#[allow(unreachable_code)] // TODO
/// Inner main function to easily pass back errors
async fn inner_main() -> Result<(), Box<dyn Error>> {
    let server: IdleServer = IdleServer {
        worlds: HashMap::default(),
        config: Config {
            packet_timeout: Duration::from_secs(5),
            ping_spacing: Duration::from_millis(50),
            default_world: String::new(),
            banned_ips: HashMap::default(),
            banned_users: HashMap::default(),
            kept_salts: 0,
            name: "OxineTesting".to_string(),
            heartbeat_url: "https://www.classicube.net/server/heartbeat".into(),
            heartbeat_retries: 5,
            heartbeat_spacing: Duration::from_millis(750),
            heartbeat_timeout: Duration::from_secs(5),
            port: 25565,
            max_players: 64,
            public: false,
        },
    };
    
    let handle = server.start().await?;
    
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    todo!()
}
