use rustc_serialize::json;
use std::io;
use std::io::{Error, ErrorKind};

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum Message {
	Login(String, String),
	Say(String),
	Move(f64, f64),
	Kill(String),
	Map(String)
}

impl Message {
	pub fn as_json(&self) -> String {
		use rustc_serialize::json;
		json::encode(&self).unwrap()
	}

	pub fn from_json(msg: &str) -> Result<Message, json::DecoderError> {
		json::decode(msg)
	}
}

pub fn next(buf: &str) -> Result<Option<(Message, String)>, json::DecoderError> {

	let remain = buf.to_string();

	//Find the null terminator that splits messages
    match buf.find("\0") {
    	Some(idx) => {
	        //Split remain_p to include the null terminator
	        let (msg_p, next_p) = remain.split_at(idx);
			//println!("Parsing: {} with {} left", msg_p, next_p);
			let r_remain = (&next_p[1..]).to_string();
			if msg_p.trim().len() != 0 {
				let new_msg = try!(Message::from_json(msg_p));
				Ok(Some((new_msg, r_remain)))
	    	} else {
				Ok(None)
	    	}
	   	},
	   	_ => Ok(None)
	}
}

/**
 * Return the error as an IO error to make error handling simpler
 */
pub fn next_io(buf: &str) -> io::Result<Option<(Message, String)>> {
	match next(buf) {
		Ok(x) => Ok(x),
		Err(e) => Err(Error::new(ErrorKind::Other, format!("JSON message decoding error {:?}", e)))
	}
}