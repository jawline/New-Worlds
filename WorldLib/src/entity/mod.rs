mod entity_type;

pub use entity::entity_type::EntityType;
use std::time::Duration;

pub struct Entity {
	pub id: usize,
	pub etype: EntityType
}

impl Entity {
	pub fn new(etype: EntityType) -> Entity {
		unsafe {
			static mut lid: usize = 0;
			lid = lid + 1;
			Entity {
				id: lid,
				etype: etype
			}
		}
	}

	pub fn update(&mut self, time: Duration) {
		println!("Ticked Entity");
	}
}