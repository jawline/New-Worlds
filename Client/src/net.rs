use std::net::TcpStream;
use std::io::{Read, Write};
use world_lib::message::{Message, next};
use std::io;
use std::str::from_utf8;
use std::io::{Error, ErrorKind};

pub struct Connection {
	pub stream: TcpStream,
	buffer: String
}

impl Connection {

	fn handle_buffer(&mut self) -> io::Result<Vec<Message>> {
		let mut buffer = Vec::new();

		'buffer: loop {
			match next(&self.buffer) {
				Ok((Some(m), remain)) => {
					self.buffer = remain;
					buffer.push(m);
				},
				Err(e) => {
					return Err(Error::new(ErrorKind::Other, format!("DecoderError {:?} in received message", e)));
				},
				_ => break 'buffer
			};
		}

		Ok(buffer)
	}

	fn buffer_self(&mut self) -> io::Result<Vec<Message>> {
		let mut buf: [u8; 4096] = [0; 4096];
		let _size = try!(self.stream.read(&mut buf));

		let fromutf = from_utf8(&buf);
		if fromutf.is_err() {
			Err(Error::new(ErrorKind::Other, "Error decoding buffered string"))
		} else {
			println!("FromUTF: {}", fromutf.unwrap());
			self.buffer = self.buffer.to_string() + &fromutf.unwrap();
			self.handle_buffer()
		}
	}

	pub fn update<T>(&mut self, callback: T) -> io::Result<()> where T: FnOnce(&Vec<Message>) -> () {
		let messages = try!(self.buffer_self());
		println!("Messages {:?}", messages);
		callback(&messages);
		Ok(())
	}

	pub fn login(&mut self, username: &str, password: &str) {
		write!(self.stream, "{}\0", Message::Login(username.to_string(), password.to_string()).as_json());
	}

	pub fn connect(server: &str) -> Connection {
		let stream = TcpStream::connect(server).unwrap();
		stream.set_nonblocking(true);
		Connection {
			stream: stream,
			buffer: "".to_string()
		}
	}
}