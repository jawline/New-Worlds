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
		let mut buf = [0u8; 256];

		while let Ok(len) = self.socket.read(&mut buf) {
			match buf.iter().take(len).position(|&x| x == b'\n') {
				Some(pos) => {
					read = read + from_utf8(&buf[0..pos]).unwrap();
					break;
				}
				_ => {
					read = read + from_utf8(&buf).unwrap();
				}
			}
    	}

    	read.trim().to_string()
	}

	pub fn write_string(&mut self, data: &str) {
		self.socket.write(data.as_bytes());
	}
}