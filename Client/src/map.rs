use tileset::Tileset;
use graphics::{Image};
use graphics::math::Matrix2d;
use conrod::backend::piston::gfx::{Transformed, G2d};
use std::default::Default;
pub use world_lib::Map;

pub fn draw(map: &Map, tiles: &Tileset, trans: Matrix2d, g: &mut G2d) {
	let image = Image::new().rect([0.0, 0.0, 64.0, 32.0]);
	for layer in &map.layers {
		for y in 0..map.height {
			for x in 0..map.width {
	    		let l_x = if y % 2 == 0 { (x as f64) * map.tile_width } else { (map.tile_width / 2.0) + (x as f64 * map.tile_width) };
		   		let l_y = (y as f64) * map.tile_height;
		   		let tile = layer[map.idx(x, y)];
				image.src_rect(tiles.src_map(tile.x, tile.y)).draw(&tiles.texture, &Default::default(), trans.trans(l_x, l_y), g);
    		}
    	}
   	}
}

pub fn get_elem(map: &Map, (x, y): (f64, f64), trans: Matrix2d) -> (usize, usize) {
	use graphics::math::transform_vec;
	let transformed = transform_vec(trans, [x, y]);
	let (mut x, y) = (transformed[0], transformed[1]);

	//Calculate y for the x offset
	let y = (y / map.tile_height) as usize;
		
	//Deal with the ridged offset
	if y % 2 == 0 {
		x -= map.tile_width / 2.0;
	}

	let x = (x / map.tile_width) as usize;

	(x, y)
}