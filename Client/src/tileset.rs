use rustc_serialize::json;
use util;
use std::path::PathBuf;
use piston_window::{Texture, Flip, G2dTexture, TextureSettings};
use conrod::backend::piston::Window;

#[derive(RustcEncodable, RustcDecodable)]
pub struct TilesetInfo {
	pub name: String,
	pub tile_width: usize,
	pub tile_height: usize
}

impl TilesetInfo {
    pub fn load(base_file: &str) -> TilesetInfo {
    	println!("{}", base_file);
        let text = util::as_string(base_file).unwrap();
        return json::decode(&text).unwrap();
    }
}

pub struct Tileset {
	pub info: TilesetInfo,
	pub texture: G2dTexture<'static>
}

impl Tileset {

	// Load the Rust logo from our assets folder.
	fn load_tileset(window: &mut Window, path: &PathBuf) -> G2dTexture<'static> {
	    Texture::from_path(&mut window.context.factory, &path, Flip::None, &TextureSettings::new()).unwrap()
	}

	pub fn new(window: &mut Window, tiles: &PathBuf, file: &str) -> Tileset {
		Tileset {
			info: TilesetInfo::load(tiles.join(&(file.to_string() + ".info")).to_str().unwrap()),
			texture: Tileset::load_tileset(window, &tiles.join(&(file.to_string() + ".png")))
		}
	}
}