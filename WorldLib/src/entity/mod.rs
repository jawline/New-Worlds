mod entity_type;

pub use entity::entity_type::EntityType;
use std::time::Duration;
use position::Position;

pub struct Entity {
	pub id: usize,
	pub etype: EntityType,
	pub position: Position
}

impl Entity {
	pub fn new(etype: EntityType, position: Position) -> Entity {
		unsafe {
			static mut lid: usize = 0;
			lid = lid + 1;
			Entity {
				id: lid,
				etype: etype,
				position: position
			}
		}
	}

	pub fn update(&mut self, time: Duration) {
		println!("Ticked Entity");
	}
}