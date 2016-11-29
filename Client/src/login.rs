use conrod;
use ui::Ids;

pub fn build_login<F>(ref mut ui: conrod::UiCell, ids: &Ids, username: &mut String, logged_in: F) where F: FnOnce() -> () {
    use conrod::{color, Labelable, widget, Colorable, Positionable, Scalar, Sizeable, Widget};

        // Our `Canvas` tree, upon which we will place our text widgets.
        widget::Canvas::new().flow_down(&[
            (ids.username_text_block, widget::Canvas::new().color(color::BLACK)),
            (ids.username_in, widget::Canvas::new().color(color::DARK_CHARCOAL)),
            (ids.username_done, widget::Canvas::new().color(color::BLACK)),
        ]).set(ids.master, ui);

        const PAD: Scalar = 20.0;

        widget::Text::new("Enter Username")
            .color(color::WHITE)
            .padded_w_of(ids.username_text_block, PAD)
            .middle_of(ids.username_text_block)
            .align_text_middle()
            .line_spacing(2.5)
            .set(ids.username_text, ui);

        for edit in widget::TextEdit::new(&username)
            .padded_w_of(ids.username_in, 20.0)
            .mid_top_of(ids.username_in)
            .align_text_x_middle()
            .line_spacing(2.5)
            .restrict_to_height(false)
            .set(ids.username_in_block, ui) {
            	*username = edit;
            }

        if widget::Button::new()
            .label("Done")
            .middle_of(ids.username_done)
            .color(color::TRANSPARENT)
            .label_color(color::WHITE)
            .set(ids.username_done_block, ui)
            .was_clicked()
            {
                logged_in();
            }
}