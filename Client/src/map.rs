use std::vec::Vec;
use tileset::Tileset;
use graphics::{Image};
use graphics::math::Matrix2d;
use conrod::backend::piston::gfx::*;
use std::default::Default;
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable, Clone, Copy)]
pub struct Tile {
	pub x: usize,
	pub y: usize
}

pub type Layer = Vec<Tile>;

#[derive(RustcEncodable, RustcDecodable)]
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

	pub fn draw(&self, tiles: &Tileset, trans: Matrix2d, g: &mut G2d) {
		let image = Image::new().rect([0.0, 0.0, 64.0, 32.0]);
		for layer in &self.layers {
	    	for y in 0..self.height {
	    		for x in 0..self.width {
	    			let l_x = if y % 2 == 0 { (x as f64) * self.tile_width } else { (self.tile_width / 2.0) + (x as f64 * self.tile_width) };
			   		let l_y = (y as f64) * self.tile_height;
			   		let tile = layer[self.idx(x, y)];
					image.src_rect(tiles.src_map(tile.x, tile.y)).draw(&tiles.texture, &Default::default(), trans.trans(l_x, l_y), g);
	    		}
	    	}
    	}
	}

	pub fn get_elem(&self, (x, y): (f64, f64), trans: Matrix2d) -> (usize, usize) {
		use graphics::math::transform_vec;
		let transformed = transform_vec(trans, [x, y]);
		let (mut x, y) = (transformed[0], transformed[1]);

		//Calculate y for the x offset
		let y = (y / self.tile_height) as usize;
		
		//Deal with the ridged offset
		if y % 2 == 0 {
			x -= self.tile_width / 2.0;
		}

		let x = (x / self.tile_width) as usize;

		(x, y)
	}

	pub fn idx(&self, x: usize, y: usize) -> usize {
		(y * self.width) + x
	}

	pub fn as_json(&self) -> String {
		json::encode(&self).unwrap()
	}
}