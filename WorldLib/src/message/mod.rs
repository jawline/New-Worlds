use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub enum Message {
	Login(String, String),
	Say(String),
	Move(f64, f64),
	Kill(String)
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