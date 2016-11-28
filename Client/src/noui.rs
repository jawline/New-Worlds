use conrod;
use ui::Ids;

pub fn no_ui(ref mut ui: conrod::UiCell, ids: &Ids) {
    use conrod::{color, widget, Colorable, Widget};
	widget::Canvas::new().color(color::TRANSPARENT).set(ids.master, ui);
}