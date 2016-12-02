use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable, Clone, Copy)]
pub struct Tile {
	pub x: usize,
	pub y: usize
}

pub type Layer = Vec<Tile>;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct Map {
	pub layers: Vec<Layer>,
	pub width: usize,
	pub height: usize,
	pub tile_width: f64,
	pub tile_height: f64
}

impl Map {

	pub fn new(w: usize, h: usize) -> Map {
		Map {
			layers: [(0..(w * h)).map(|_| Tile {x: 0, y: 0}).collect()].to_vec(),
			width: w,
			height: h,
			tile_width: 64.0,
			tile_height: 16.0
		}
	}

	pub fn idx(&self, x: usize, y: usize) -> usize {
		(y * self.width) + x
	}

	pub fn as_json(&self) -> String {
		json::encode(&self).unwrap()
	}

	pub fn from_json(data: &str) -> Map {
		json::decode(data).unwrap()
	}
}