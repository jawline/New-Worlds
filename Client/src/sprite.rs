use piston_window::{G2dTexture};
use graphics::Image;
use graphics::math::Matrix2d;
use conrod::backend::piston::gfx::{Transformed, G2d};
use world_lib::Entity;

pub fn draw(entity: &Entity, texture: &G2dTexture<'static>, trans: Matrix2d, g: &mut G2d) {
	Image::new().rect([0.0, 0.0, 64.0, 32.0]).src_rect([0.0, 0.0, 56.0, 56.0]).draw(texture, &Default::default(), trans.trans(entity.pos.x, entity.pos.y), g);
}