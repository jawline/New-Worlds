pub struct Zone {
	pub id: usize,
	pub name: String,
	pub desc: String
}

impl Zone {
	pub fn load(id: usize) -> Option<Zone> {
		match id {
			0 => Some(Zone{
				id: id,
				name: "The Infinate Room".to_string(),
				desc: "An infinate room with nothing for infinity in all directions".to_string()
			}),
			_ => None
		}
	}
}