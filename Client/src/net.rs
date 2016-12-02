use std::net::TcpStream;
use std::io::{Read, Write};
use world_lib::message::{Message, next};
use std::io;

pub struct Connection {
	pub stream: TcpStream,
	buffer: Vec<u8>
}

impl Connection {

	fn handle_buffer(&mut self) -> io::Result<Vec<Message>> {
		let mut buffer = Vec::new();

		while let Some((msg, remain)) = try!(next(&self.buffer)) {
			buffer.push(msg);
			self.buffer = remain;
		}

		Ok(buffer)
	}

	fn buffer_self(&mut self) -> io::Result<Vec<Message>> {
		let mut buf: [u8; 4096] = [0; 4096];
		let size = try!(self.stream.read(&mut buf));
		self.buffer = self.buffer.iter().chain(&buf[0..size]).map(|&x| x).collect();
		self.handle_buffer()
	}

	pub fn update<T>(&mut self, callback: T) -> io::Result<()> where T: FnOnce(&Vec<Message>) -> () {
		let messages = try!(self.buffer_self());
		callback(&messages);
		Ok(())
	}

	pub fn send(&mut self, message: &Message) -> io::Result<()> {
		write!(self.stream, "{}\0", message.as_json())
	}

	pub fn login(&mut self, username: &str, password: &str) -> io::Result<()> {
		self.send(&Message::Login(username.to_string(), password.to_string()))
	}

	pub fn connect(server: &str) -> Connection {
		let stream = TcpStream::connect(server).unwrap();
		stream.set_nonblocking(true);
		Connection {
			stream: stream,
			buffer: Vec::new()
		}
	}
}