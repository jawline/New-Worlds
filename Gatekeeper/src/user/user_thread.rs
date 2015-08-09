use nwshared::Client;
use user::User;
use std::io::{Read, Write};

pub fn user_thread<T: Read + Write>(user: &mut User, client: &mut Client<T>) {
	client.write_string(&format!("Welcome {}", user.name));
}