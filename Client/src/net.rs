use std::net::TcpStream;
use std::io::{Read, Write};

pub struct Connection {
	pub stream: TcpStream
}

impl Connection {

	fn consume_ack(stream: &mut TcpStream) {
		let mut buf = [0; 8096];
		while stream.read(&mut buf).is_err() {}
	}

	pub fn connect(server: &str, username: &str) -> Connection {
		let mut stream = TcpStream::connect(server).unwrap();
		stream.set_nonblocking(true);
		write!(stream, "{}", username);

		Connection::consume_ack(&mut stream);

		write!(stream, "say Hi\n");

		Connection {
			stream: stream
		}
	}
}