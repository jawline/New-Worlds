#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate graphics;
extern crate rustc_serialize;
extern crate piston_window;
extern crate world_lib;

mod ui;
mod login;
mod util;
mod tileset;
mod noui;
mod fonts;
mod assets;
mod map;
mod net;

use map::{Map};
use login::*;
use graphics::math::{identity, Matrix2d};

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::{UpdateEvent};
use piston_window::{PressEvent, MouseCursorEvent, MouseButton, clear, ReleaseEvent, Button, Key, G2d, Transformed};
use world_lib::message::Message;

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

fn build_window() -> (Window, WindowEvents, conrod::Ui, ui::Ids) {
    let opengl = OpenGL::V3_2;
    let window = piston::window::WindowSettings::new("New Worlds", [WIDTH, HEIGHT]).opengl(opengl).exit_on_esc(true).build().unwrap();
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    let ids = ui::Ids::new(ui.widget_id_generator());
    (window, WindowEvents::new(), ui, ids)
}

fn main() {

    let (mut window, mut events, mut ui, ids) = build_window();
    fonts::setup(&mut ui);

    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut user: String = "John Doe".to_string();
    let mut logged_in = false;

    let tiles = tileset::Tileset::new(&mut window, &assets::tiles(), "grass");

    let mut x_off = 0.0;
    let mut y_off = 0.0;

    let mut l_press = false;
    let mut r_press = false;
    let mut up = false;
    let mut down = false;
    let mut zoom = false;
    let mut zoom_out = false;
    let mut scale = 1.0;
    let mut cursor = (0.0, 0.0);
    
    let mut curmap: Option<Map> = None;
    let mut conn = net::Connection::connect("127.0.0.1:15340");

    fn build_transform(initial: Matrix2d, (x_off, y_off): (f64, f64), scale: f64) -> Matrix2d {
        initial.scale(scale, scale).trans(-x_off, -y_off)
    }

    fn build_inverse(initial: Matrix2d, (x_off, y_off): (f64, f64), scale: f64) -> Matrix2d {
        initial.trans(x_off, y_off).scale(scale, scale)
    }

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        event.mouse_cursor(|x, y| {
            cursor = (x,y);
        });

        if let Some(button) = event.press_args() {
            
            if button == Button::Keyboard(Key::Left) {
                l_press = true;
            } else if button == Button::Keyboard(Key::Right) {
                r_press = true;
            } else if button == Button::Keyboard(Key::Up) {
                up = true;
            } else if button == Button::Keyboard(Key::Down) {
                down = true;
            } else if button == Button::Keyboard(Key::J) {
                zoom = true;
            } else if button == Button::Keyboard(Key::K) {
                zoom_out = true;
            }

        } else if let Some(button) = event.release_args() {
            
            if button == Button::Keyboard(Key::Left) {
                l_press = false;
            }

            if button == Button::Keyboard(Key::Right) {
                r_press = false;
            }

            if button == Button::Keyboard(Key::Up) {
                up = false;
            }

            if button == Button::Keyboard(Key::Down) {
                down = false;
            }

            if button == Button::Keyboard(Key::J) {
                zoom = false;
            }

            if button == Button::Keyboard(Key::K) {
                zoom_out = false;
            }

            if button == Button::Mouse(MouseButton::Left) {

                match curmap {
                    Some(ref mut map) => {
                        let (x, y) = map::get_elem(map, cursor, build_inverse(identity(), (x_off, y_off), scale));
                        println!("{} {}", x, y);
                        if x < map.width && y < map.height {
                            let idx = map.idx(x,y);
                            map.layers[0][idx].y = 1;
                        }
                        conn.send(&Message::Map(map.as_json()));
                    },
                    _ => {}
                }
            }
        }

        if l_press {
            x_off -= 1.0;
        } else if r_press {
            x_off += 1.0;
        }

        if up {
            y_off -= 1.0;
        } else if down {
            y_off += 1.0;
        }

        if zoom {
            scale += 0.1;
        } else if zoom_out {
            scale -= 0.1;
        }

        if scale < 1.0 {
            scale = 1.0;
        }

        /* Convert unused events to conrod events */ 
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
        	if !logged_in {
        		build_login(ui.set_widgets(), &ids, &mut user, |username| {
                    logged_in = true;
                    conn.login(username, "test");
                });
        	} else {
        		noui::no_ui(ui.set_widgets(), &ids);
        	}
        });

        match conn.update(|messages| {
            println!("Message {:?}", messages);
            for message in messages {
                match message {
                    &Message::Map(ref data) => {
                        println!("Loading map from MapData");
                        curmap = Some(Map::from_json(data));
                    },
                    _ => {}
                }
            }
        }) {
            Err(_) => { /* TODO: Conn error handling */ },
            _ => {}
        }

        window.draw_2d(&event, |c, g: &mut G2d| {

            clear([0.0,0.0,0.0,0.0], g);

            match curmap {
                Some(ref map) => {
                    map::draw(map, &tiles, build_transform(c.transform, (x_off, y_off), scale), g);
                },
                None => { /* No map to draw */ }
            }

            if !logged_in {
                if let Some(primitives) = ui.draw_if_changed() {
                    fn texture_from_image<T>(img: &T) -> &T { img };
                    piston::window::draw(c, g, primitives,
                  		&mut text_texture_cache,
                        &image_map,
                        texture_from_image);
                }

                ui.needs_redraw();
            }

            std::thread::sleep(std::time::Duration::from_millis(1))
        });
    }

}