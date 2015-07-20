extern crate nwshared;

use nwshared::Client;
use std::net::TcpStream;

fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:19111").unwrap();
    let mut client = Client::new(socket);
    client.write_string("Albion\n");
}