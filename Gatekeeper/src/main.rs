extern crate nwshared;

mod user;
mod zone;
mod map;

use std::net::TcpListener;
use std::thread::spawn;
use map::Map;
use zone::Zone;
use nwshared::Client;
use std::sync::Arc;

fn user_server(map: &Arc<Map>) {
    let listener = TcpListener::bind("0.0.0.0:15340").unwrap();
	for client in listener.incoming() {
		let stream = client.unwrap();
		let client_map = map.clone();
	    spawn(move || {
	    	let mut client = Client::new(stream);

	    	client.write_string("Welcome, Please enter your name\n");
	    	client.write_string("Name: ");
	    	
	    	let mut user = user::User::load(client.read_string(), client_map);
	    	client.write_string("Logging In\n");
	    	
	    	user::user_thread(&mut user, &mut client);
	    });
	}
}

fn main() {

	let mut map = Map::new("The Infinate World", "An infinate expanse used to test out ideas relating to the origins of the universe");
	map.add_zone(Zone::load(0).unwrap());

	let map_rc = Arc::new(map);

    let user_server = spawn(move || {
    	user_server(&map_rc);
    });

    user_server.join().unwrap();
}