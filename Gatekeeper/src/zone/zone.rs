pub struct Zone {
	pub id: usize,
	pub name: String,
	pub desc: String
}

impl Zone {
	pub fn new(id: usize, name: &str, desc: &str) -> Zone {
		Zone{
			id: id,
			name: name.to_string(),
			desc: desc.to_string()
		}
	}
}