use entity::Entity;
use std::time::Duration;

pub struct World {
	entities: Vec<Entity>
}

impl World {
	pub fn new() -> World {
		World {
			entities: Vec::new()
		}
	}

	pub fn update(&mut self, utime: Duration) {
		for entity in &mut self.entities {
			entity.update(utime);
		}
	}

	pub fn serialize(&self) -> String {
		"none".to_string()
	}
}