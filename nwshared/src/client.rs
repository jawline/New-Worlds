use std::io::{Read, Write};
use std::str::from_utf8;

pub struct Client<T: Read + Write> {
	pub socket: T
}

impl <T: Read + Write> Client<T> {
	pub fn new(socket: T) -> Client<T> {
		Client{socket: socket}
	}

	pub fn read_string(&mut self) -> String {
		let mut read = String::new();
		let mut buf = [0u8; 512];

		while let Ok(len) = self.socket.read(&mut buf) {
    		read = read + from_utf8(&buf).unwrap();
    		if buf.iter().take(len).find(|&x| *x == b'\n').is_some() {
    			break;
    		}
    	}

    	read
	}

	pub fn write_string(&mut self, data: &str) {
		self.socket.write(data.as_bytes());
	}
}