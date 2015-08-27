use std::sync::Arc;
use map::Map;

pub struct User {
	pub name: String,
	pub current_zone: usize,
	pub map: Arc<Map>
}

impl User {
	pub fn load(user_name: &str, map: Arc<Map>) -> User {
		User {
			name: user_name.to_string(),
			current_zone: map.start_zone,
			map: map
		}
	}

	pub fn current_zone(&self) -> String {
		self.map.zones[self.current_zone].name.clone()
	}
}