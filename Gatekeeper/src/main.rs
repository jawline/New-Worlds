extern crate nwshared;
extern crate mio;

#[macro_use] extern crate log;
extern crate env_logger;

mod user;
mod zone;
mod map;
mod connection;
mod server;

use std::net::SocketAddr;
use std::str::FromStr;

use mio::*;
use mio::tcp::*;

use server::Server;

fn main() {

    let mut map = map::Map::new("The Infinate World", "An empty infinate expanse some think was used for testing");
    map.add_zone(zone::Zone::load(0).unwrap());

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