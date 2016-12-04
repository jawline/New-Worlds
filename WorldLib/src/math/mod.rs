pub type Vec2d = (f64, f64);

pub trait Vector {
	fn add(&self, other: &Vec2d) -> Vec2d;
	fn mul(&self, scalar: f64) -> Vec2d;
	fn neg(&self) -> Vec2d;
}

impl Vector {
	pub fn sub(&self, other: &Vec2d) -> Vec2d {
		self.add(&other.neg())
	}
}

impl Vector for Vec2d {
	fn add(&self, other: &Vec2d) -> Vec2d {
		let &(s_x, s_y) = self;
		let &(o_x, o_y) = other;
		(s_x + o_x, s_y + o_y)
	}

	fn mul(&self, scalar: f64) -> Vec2d {
		let &(s_x, s_y) = self;
		(s_x * scalar, s_y * scalar)
	}

	fn neg(&self) -> Vec2d {
		let &(s_x, s_y) = self;
		(-s_x, -s_y)
	}
}