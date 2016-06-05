pub struct Position {
	pub x: f64,
	pub y: f64,
	pub z: f64
}

impl Position {
	pub fn new(x: f64, y: f64, z: f64) -> Position {
		Position {
			x: x,
			y: y,
			z: z
		}
	}
}