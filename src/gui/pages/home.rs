use iced::{Element, Theme};
use iced::widget::{
    column, text
};
use super::super::message::Event;


pub fn view() -> Element<'static, Event, Theme> {
    column![
        text("欢迎使用SysGuard")
    ].into()
}