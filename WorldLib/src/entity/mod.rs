use std::time::Duration;
use math::Vec2d;

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum EntityType {
	Scene,
	Character
}

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct Entity {
	pub id: usize,
	pub t: EntityType,
	pub pos: Vec2d
}

impl Entity {
	pub fn new(etype: EntityType, position: Vec2d) -> Entity {
		unsafe {
			static mut lid: usize = 0;
			lid = lid + 1;
			Entity {
				id: lid,
				t: etype,
				pos: position
			}
		}
	}

	pub fn update(&mut self, time: Duration) {}
}