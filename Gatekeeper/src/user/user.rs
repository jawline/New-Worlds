pub struct User {
	pub name: String,
	pub current_zone: usize
}

impl User {
	pub fn load(user_name: &str, zone: usize) -> User {
		User {
			name: user_name.to_string(),
			current_zone: zone
		}
	}

	pub fn set_name(&mut self, user_name: &str) {
		self.name = user_name.to_string();
	}
}