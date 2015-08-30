use zone::Zone;

pub struct Map {
	pub name: String,
	pub desc: String,
	pub start_zone: usize,
	pub zones: Vec<Zone>
}

impl Map {

	pub fn new(name: &str, desc: &str, start_zone: usize) -> Map {
		Map {
			name: name.to_string(),
			desc: desc.to_string(),
			start_zone: start_zone,
			zones: Vec::new()
		}
	}

	pub fn add_zone(&mut self, zone: Zone) {
		self.zones.push(zone);
	}

	pub fn valid_zone_id(&self, zone_id: usize) -> bool {
		zone_id < self.zones.len()
	}

	pub fn find_zone_from_name(&self, name: &str) -> Option<usize> {
		for i in 0..self.zones.len() {
			if self.zones[i].name == name {
				return Some(i);
			}
		}
		None
	}
}