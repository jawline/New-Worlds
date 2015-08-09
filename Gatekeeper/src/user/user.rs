pub struct User {
	pub name: String
}

impl User {
	pub fn load(user_name: String) -> User {
		User {
			name: user_name
		}
	}
}