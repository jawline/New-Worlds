extern crate mio;
extern crate world_lib;

#[macro_use] extern crate log;
extern crate env_logger;
extern crate rustc_serialize;

mod user;
mod connection;
mod server;

use std::net::SocketAddr;
use std::str::FromStr;

use mio::*;
use mio::tcp::*;

use server::Server;

use world_lib::Map;

fn main() {

    env_logger::init().ok().expect("Failed to init logger");

    let addr: SocketAddr = FromStr::from_str("127.0.0.1:15340")
        .ok().expect("Failed to parse host:port string");

    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");

    let mut event_loop = EventLoop::new().ok().expect("Failed to create event loop");

    let mut server = Server::new(sock, Map::new(16, 32));
    server.register(&mut event_loop).ok().expect("Failed to register server with event loop");

    info!("Even loop starting...");
    event_loop.run(&mut server).ok().expect("Failed to start event loop");
}