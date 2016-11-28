use std::vec::Vec;
use tileset::Tileset;
use graphics::{Image};
use conrod::backend::piston::gfx::*;
use std::default::Default;
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable, Clone, Copy)]
pub struct Tile {
	x: usize,
	y: usize
}

pub type Layer = Vec<Tile>;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Map {
	pub layers: Vec<Layer>,
	pub width: usize,
	pub height: usize
}

impl Map {

	pub fn new(w: usize, h: usize) -> Map {
		Map {
			layers: [(0..(w * h)).map(|_| Tile {x: 0, y: 0}).collect()].to_vec(),
			width: w,
			height: h
		}
	}

	pub fn draw(&self, tiles: &Tileset, trans: [[f64; 3]; 2], g: &mut G2d) {
		let image = Image::new().rect([0.0, 0.0, 64.0, 32.0]);
		for layer in &self.layers {
	    	for y in 0..self.height {
	    		for x in 0..self.width {
	    			let l_x = if y % 2 == 0 { (x as f64) * 64.0 } else { 32.0 + (x as f64 * 64.0) };
			   		let l_y = (y as f64) * 16.0;
			   		let tile = layer[(y * self.width) + x];
					image.src_rect(tiles.src_map(tile.x, tile.y)).draw(&tiles.texture, &Default::default(), trans.trans(l_x, l_y), g);
	    		}
	    	}
    	}
	}

	pub fn as_json(&self) -> String {
		json::encode(&self).unwrap()

	}
}