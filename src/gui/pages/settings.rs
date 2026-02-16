use iced::{Element, Theme};
use iced::widget::{
    text
};
use super::super::message::Event;

pub fn view() -> Element<'static, Event, Theme> {
    text("settings").into()
}