extern crate nwshared;

mod user;
mod zone;
mod map;

use std::net::TcpListener;
use std::thread::spawn;
use map::Map;
use zone::Zone;
use nwshared::Client;

fn user_server() {
    let listener = TcpListener::bind("0.0.0.0:15340").unwrap();
	for client in listener.incoming() {
		let stream = client.unwrap();
	    spawn(move || {
	    	let mut client = Client::new(stream);

	    	client.write_string("Welcome, Please enter your name\n");
	    	client.write_string("Name: ");
	    	
	    	let mut user = user::User::load(client.read_string());
	    	client.write_string("Logging In\n");
	    	
	    	user::user_thread(&mut user, &mut client);
	    });
	}
}

fn main() {

	let mut map = Map::new("The Infinate World");
	map.add_zone(Zone::load(0).unwrap());

    let user_server = spawn(move || {
    	user_server();
    });

    user_server.join().unwrap();
}