use zone::Zone;

pub struct Map {
	pub name: String,
	pub zones: Vec<Zone>
}

impl Map {
	pub fn new(name: &str) -> Map {
		Map {
			name: name.to_string(),
			zones: Vec::new()
		}
	}

	pub fn add_zone(&mut self, zone: Zone) {
		self.zones.push(zone);
	}
}