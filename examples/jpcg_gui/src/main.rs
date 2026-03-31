mod gui;
mod log;

use crate::gui::Counter;
fn main() -> iced::Result {
    iced::application(Counter::default, Counter::update, Counter::view)
        .title("JX3PVP Calculator")
        .font(include_bytes!("../res/SourceHanSansSC-Regular.otf"))
        .default_font(iced::Font::with_name("思源黑体"))
        .window(iced::window::Settings {
            size: iced::Size::new(1520.0, 450.0),
            ..iced::window::Settings::default()
        })
        .run()
}
