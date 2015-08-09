extern crate nwshared;

mod user;

use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread::spawn;
use std::str::from_utf8;
use std::vec::Vec;
use std::sync::{Arc, Mutex};
use nwshared::Client;

fn user_server() {
    let listener = TcpListener::bind("0.0.0.0:15340").unwrap();
	for client in listener.incoming() {
		let mut stream = client.unwrap();
	    spawn(move || {

	    	let mut client = Client::new(stream);

	    	client.write_string("Welcome, Please enter your name\n");
	    	client.write_string("Name: ");
	    	
	    	let mut user = user::User {
	    		name: client.read_string()
	    	};
	    	
	    	user::user_thread(&mut user, &mut client);
	    });
	}
}

fn main() {

    let user_server = spawn(move || {
    	user_server();
    });

    user_server.join();
}