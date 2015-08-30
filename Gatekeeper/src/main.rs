extern crate mio;

#[macro_use] extern crate log;
extern crate env_logger;

mod user;
mod zone;
mod map;
mod connection;
mod server;
pub mod help;

use zone::Zone;
use map::Map;

use std::net::SocketAddr;
use std::str::FromStr;

use mio::*;
use mio::tcp::*;

use server::Server;

fn main() {

    let mut map = Map::new("The Infinite World", "An empty infinite expanse some think was used for testing", 1);

    map.add_zone(Zone::new(0, "The Infinite", ""));
    map.add_zone(Zone::new(1, "The Spire", ""));

    map.add_zone(Zone::new(2, "Lower Spire", ""));
    map.add_zone(Zone::new(3, "Lower Spire - Trade Quarters", ""));
    map.add_zone(Zone::new(4, "Lower Spire - Arena", ""));
    
    map.add_zone(Zone::new(5, "Challenge Zone", ""));
    map.add_zone(Zone::new(6, "Bubbling Sewers", ""));
    map.add_zone(Zone::new(7, "Salty Dungeon", ""));
    map.add_zone(Zone::new(8, "Demon Alter", ""));

    env_logger::init().ok().expect("Failed to init logger");

    let addr: SocketAddr = FromStr::from_str("127.0.0.1:15340")
        .ok().expect("Failed to parse host:port string");

    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");

    let mut event_loop = EventLoop::new().ok().expect("Failed to create event loop");

    let mut server = Server::new(sock, map);
    server.register(&mut event_loop).ok().expect("Failed to register server with event loop");

    info!("Even loop starting...");
    event_loop.run(&mut server).ok().expect("Failed to start event loop");
}