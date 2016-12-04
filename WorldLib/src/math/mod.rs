#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct Vec2d {
	pub x: f64,
	pub y: f64
}

impl Vec2d {
	pub fn new(x: f64, y: f64) -> Vec2d {
		Vec2d {
			x: x,
			y: y
		}
	}

	pub fn add(&self, other: &Vec2d) -> Vec2d {
		Vec2d {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}

	pub fn mul(&self, scalar: f64) -> Vec2d {
		Vec2d {
			x: self.x * scalar,
			y: self.y * scalar
		}
	}

	pub fn neg(&self) -> Vec2d {
		Vec2d {
			x: -self.x,
			y: -self.y
		}
	}

	pub fn sub(&self, other: &Vec2d) -> Vec2d {
		self.add(&other.neg())
	}
}