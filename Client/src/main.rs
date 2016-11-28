#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate graphics;
extern crate opengl_graphics;
extern crate rustc_serialize;
extern crate piston_window;

mod ui;
mod login;
mod util;
mod tileset;
mod noui;
mod fonts;
mod assets;
mod map;

use map::Map;
use login::*;

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;
use graphics::{Image};
use std::default::Default;
use piston_window::Transformed;

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

fn build_window() -> Window {
    let opengl = OpenGL::V3_2;
	piston::window::WindowSettings::new("New Worlds", [WIDTH, HEIGHT])
        .opengl(opengl).exit_on_esc(true).build().unwrap()
}

fn main() {

    let mut window = build_window();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = ui::Ids::new(ui.widget_id_generator());

    fonts::setup(&mut ui);

    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut user: String = "John Doe".to_string();
    let mut logged_in = false;

    let tiles = tileset::Tileset::new(&mut window, &assets::tiles(), "grass");
    let map = Map::new(64, 64);

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
        	if !logged_in {
        		build_login(ui.set_widgets(), &ids, &mut user, &mut logged_in);
        	} else {
        		noui::no_ui(ui.set_widgets(), &ids);
        	}
        });

        window.draw_2d(&event, |c, g| {

        	piston_window::clear([0.5, 1.0, 0.5, 1.0], g);

            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                piston::window::draw(c, g, primitives,
              		&mut text_texture_cache,
                    &image_map,
                    texture_from_image);
            }

    		let image = Image::new().rect([0.0, 0.0, 64.0, 32.0]).src_rect([0.0, 32.0, 64.0, 32.0]);

    		for y in 0..64 {
    			for x in 0..64 {
    				let l_x = if y % 2 == 0 { (x as f64) * 64.0 } else { 32.0 + (x as f64 * 64.0) };
		    		let l_y = (y as f64) * 16.0;
		    		image.draw(&tiles.texture, &Default::default(), c.transform.trans(l_x, l_y), g);
    			}
    		}

            ui.needs_redraw();
        });
    }

}