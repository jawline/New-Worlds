use std::io;
use std::result::Result;
use std::error::Error;

pub fn to_io<Q, T>(msg: Result<Q, T>) -> io::Result<Q> where T: Error {
	match msg {
		Ok(e) => Ok(e),
		Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))
	}
}