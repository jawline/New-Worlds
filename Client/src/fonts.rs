use conrod;
use assets;

pub fn setup(ui: &mut conrod::Ui) {
	let noto_sans = assets::path().join("fonts/NotoSans");
    let regular = ui.fonts.insert_from_file(noto_sans.join("NotoSans-Regular.ttf")).unwrap();
    ui.theme.font_id = Some(regular);
}