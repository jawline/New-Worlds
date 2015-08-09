use nwshared::Client;
use user::User;
use user::help;
use std::io::{Read, Write};

pub fn user_thread<T: Read + Write>(user: &mut User, client: &mut Client<T>) {
	client.write_string(&format!("Welcome {}\n", user.name));

	loop {
		client.write_string("Command: ");
		let command = client.read_string();

		match &command as &str {
			"help" => {
				client.write_string(help::get_help_text());
			},
			"say" => {
				client.write_string("What do you want to say?");
			},
			"world" => {
				client.write_string("You are in a nameless void\n");
			},
			"zone" => {
				client.write_string("You are in an empty universe, with nothing streching off in all directions\n");
			},
			_ => {
				client.write_string(&format!("Error, unhandled command {}\n", command));
			}
		}
	}
}