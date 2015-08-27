use zone::Zone;

pub struct Map {
	pub name: String,
	pub desc: String,
	pub start_zone: usize,
	pub zones: Vec<Zone>
}

impl Map {
	pub fn new(name: &str, desc: &str) -> Map {
		Map {
			name: name.to_string(),
			desc: desc.to_string(),
			start_zone: 0,
			zones: Vec::new()
		}
	}

	pub fn add_zone(&mut self, zone: Zone) {
		self.zones.push(zone);
	}

	pub fn valid_zone_id(&self, zone_id: usize) -> bool {
		zone_id < self.zones.len()
	}
}