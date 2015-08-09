use std::sync::Arc;
use map::Map;

pub struct User {
	pub name: String,
	pub current_zone: usize,
	pub map: Arc<Map>
}

impl User {
	pub fn load(user_name: String, map: Arc<Map>) -> User {
		User {
			name: user_name,
			current_zone: map.start_zone,
			map: map
		}
	}
}