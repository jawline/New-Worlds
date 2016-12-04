use rustc_serialize::json;
use entity::{Entity, EntityID};
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

	pub fn update_or_insert(&mut self, entity: &Entity) {
		if let Some(pos) = self.entities.iter().position(|x| x.id == entity.id) {
			self.entities[pos] = entity.clone();
		} else {
			self.entities.push(entity.clone());
		}
	}

	pub fn remove(&mut self, id: EntityID) {
		if let Some(pos) = self.entities.iter().position(|x| x.id == id) {
			self.entities.remove(pos);
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