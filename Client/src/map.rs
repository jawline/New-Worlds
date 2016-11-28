use std::vec::Vec;
use piston_window::Transformed;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Tile {
	x: usize,
	y: usize
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct Map {
	pub tiles: Vec<Tile>,
	pub width: usize,
	pub height: usize
}

impl Map {
	pub fn new(w: usize, h: usize) -> Map {
		Map {
			tiles: (0..(w * h)).map(|_| Tile {x: 0, y: 0}).collect(),
			width: w,
			height: h
		}
	}

	pub fn draw(trans: [f64; 4]) {

	}
}