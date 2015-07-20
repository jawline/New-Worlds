extern crate nwshared;

use std::net::TcpListener;
use std::io::Write;
use std::io::Read;
use std::thread::spawn;
use std::str::from_utf8;
use std::vec::Vec;
use std::sync::{Arc, Mutex};
use nwshared::Client;

pub struct ZoneInfo {
	pub name: String,
	pub port: String
}

fn user_server(zone_list: Arc<Mutex<Vec<ZoneInfo>>>) {
    let listener = TcpListener::bind("0.0.0.0:15340").unwrap();
	for client in listener.incoming() {
		let mut stream = client.unwrap();
		let local_zlist = zone_list.clone();
	    spawn(move || {
	    	let mut client = Client::new(stream);

	    	client.write_string("Welcome to Gatekeeper\n");
	    	client.write_string("Username: ");

	    	let name = client.read_string();
	    	client.write_string(&format!("Welcome {}", name));

	    	let zl = local_zlist.lock().unwrap();

	    	client.write_string("Active Zones\n");
	    	
	    	for item in zl.iter() {
	    		client.write_string(&format!("{}\n", item.name));
	    	}

	    	client.write_string("END\n");
	    });
	}
}

fn zone_thread<T: Read + Write>(client: &mut Client<T>, zone_list: Arc<Mutex<Vec<ZoneInfo>>>) {
	client.write_string("GK\n");

	let zserver_name = client.read_string();
	client.write_string("ACK\n");

	let zserver_port = client.read_string();
	client.write_string("ACK\n");

	println!("Register zone server {}", zserver_name.trim());
	
	zone_list.lock().unwrap().push(ZoneInfo{
		name: format!("{}", zserver_name.trim()),
		port: format!("{}", zserver_port.trim())
	});	
}

fn zone_server(zone_list: Arc<Mutex<Vec<ZoneInfo>>>) {
    let listener = TcpListener::bind("127.0.0.1:19111").unwrap();
	for client in listener.incoming() {
		let mut stream = client.unwrap();
		let zone_list_thread = zone_list.clone();
		let mut client = Client::new(stream);
	    spawn(move || {
	    	zone_thread(&mut client, zone_list_thread);
	    });
	}
}

fn main() {

	let zone_list = Arc::new(Mutex::new(Vec::<ZoneInfo>::new()));
	let zone_list_copy = zone_list.clone();

    let user_server = spawn(move || {
    	user_server(zone_list);
    });

    let zone_server = spawn(move || {
    	zone_server(zone_list_copy);
    });

    user_server.join();
    zone_server.join();
}