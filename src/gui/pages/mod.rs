mod home;
mod process;
mod settings;

use iced::{Element, Theme};
use crate::gui::app::SysGuard;
use super::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Home,
    Process,
    Settings,
}
impl Page {
    pub fn page_idx(&self) -> usize {
        match self {
            Page::Home => 0,
            Page::Process => 1,
            Page::Settings => 2,
        }
    }
}

pub fn view<'a>(
    page: &Page,
    sys_guard: &'a SysGuard,
) -> Element<'a, Event, Theme> {
    match page {
        Page::Home => home::view(),
        Page::Process => process::view(sys_guard),
        Page::Settings => settings::view()
    }
}