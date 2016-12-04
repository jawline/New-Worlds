use rustc_serialize::json;
use entity::Entity;
use map::Map;
use std::time::Duration;

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct World {
	pub map: Map,
	pub entities: Vec<Entity>
}

impl World {

	pub fn new(map: Map) -> World {
		World {
			entities: Vec::new(),
			map: map
		}
	}

	pub fn update(&mut self, utime: Duration) {
		for entity in &mut self.entities {
			entity.update(utime);
		}
	}

	pub fn from_json(t: &str) -> World {
		json::decode(t).unwrap()
	}

	pub fn as_json(&self) -> String {
		json::encode(self).unwrap()
	}
}