use iced::{Element, Theme};
use iced::widget::{
    text
};
use super::super::event::Event;

pub fn view() -> Element<'static, Event, Theme> {
    text("settings").into()
}