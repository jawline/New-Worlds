use rustc_serialize::json;
use std::io;
use std::result::Result;
use std::error::Error;
use std::str::from_utf8;
use utils::to_io;

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

pub fn next(buf: &Vec<u8>) -> io::Result<Option<(Message, Vec<u8>)>> {

	let remain = buf.to_vec();

	//Find the null terminator that splits messages
    match buf.iter().position(|&x| x == 0) {
    	Some(idx) => {
	        //Split remain_p to include the null terminator
	        let (msg_p, next_p) = remain.split_at(idx);
			let r_remain = &next_p[1..];

			let text = try!(to_io(from_utf8(msg_p))).trim();
			if text.len() != 0 {
				let new_msg = try!(to_io(Message::from_json(text.trim())));
				Ok(Some((new_msg, r_remain.to_vec())))
			} else {
				Ok(None)
			}
	   	},
	   	_ => Ok(None)
	}
}