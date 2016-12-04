use std::time::Duration;
use rustc_serialize::json;
use math::Vec2d;

pub type EntityID = usize;

pub fn null_id() -> EntityID {
	0
}

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum EntityType {
	Scene,
	Character
}

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct Entity {
	pub id: usize,
	pub t: EntityType,
	pub pos: Vec2d,
	pub size: Vec2d
}

impl Entity {
	pub fn new(etype: EntityType, position: Vec2d, size: Vec2d) -> Entity {
		unsafe {
			static mut lid: usize = 0;
			lid = lid + 1;
			Entity {
				id: lid,
				t: etype,
				pos: position,
				size: size
			}
		}
	}

	pub fn from_json(t: &str) -> Entity {
		json::decode(t).unwrap()
	}

	pub fn as_json(&self) -> String {
		json::encode(self).unwrap()
	}

	pub fn update(&mut self, time: Duration) {}
}