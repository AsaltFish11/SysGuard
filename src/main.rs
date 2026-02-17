use iced::Size;
use iced::window::Settings;
mod gui;
mod constant;
mod utils;
mod process;

fn main() -> iced::Result {
    let mut app_window_setting = Settings::default();
    app_window_setting.min_size = Some(Size::new(800.0, 600.0));
    gui::app::start(app_window_setting)
}