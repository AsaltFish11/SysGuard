use iced::{Element, Theme, color, Background, Fill, Alignment};
use iced::border::Radius;
use iced::Length::Fixed;
use iced::widget::{Button, button, column, container, row};
use iced::window::Settings;
use super::pages::{
    view, Page
};
use super::super::constant::*;
use super::message::Event;
use super::super::utils::process::*;
use super::super::utils::handle::list_pagination;


pub struct SysGuard {
    current_page: Page,
    current_page_button: usize,
    pub processes: Vec<Vec<ProcessEntry>>,
    pub current_page_idx: usize,
    pub select_proc: Option<usize>,
}
fn create_nav_button(
    nav_button_idx: usize,
    on_press: Event,
    current_page_idx: usize,
) -> Button<'static, Event, Theme> {
    let is_select = nav_button_idx == current_page_idx;

    button(NAV_BUTTON_TEXT[nav_button_idx])
        .on_press(on_press)
        .width(Fixed(NAV_BUTTON_WIDTH - 8.0))
        .style(move |theme: &Theme, status| {
            let mut style = button::primary(theme, status);

            let (normal_bg, hover_bg, pressed_bg) = if is_select {
                (
                    color!(71, 194, 253), // 正常
                    color!(71, 194, 253), // 悬停
                    color!(41, 144, 203), // 按下
                )
            } else {
                (
                    color!(18, 18, 18),   // 正常
                    color!(32, 67, 87),   // 悬停
                    color!(41, 144, 203), // 按下
                )
            };

            style.background = Some(Background::Color(match status {
                button::Status::Hovered => hover_bg,
                button::Status::Pressed => pressed_bg,
                _ => normal_bg,
            }));

            style.border.color = color!(61, 174, 233);
            style.border.width = if status == button::Status::Hovered { 1.0 } else { 0.0 };
            style.border.radius = Radius::new(4);
            style
        })
}
impl SysGuard {
    pub fn new() -> Self {
        SysGuard {
            current_page: Page::Home,
            current_page_button: 0,
            processes: list_pagination(fetch_processes(), PROC_PAGE_SIZE),
            current_page_idx: 0,
            select_proc: None,
        }
    }

    pub fn update(&mut self, msg: Event) {
        match msg {
            Event::NavTo(page) => {
                self.current_page = page;
                self.current_page_button = page.page_idx();
            },
            Event::Refresh => {
                self.processes = list_pagination(fetch_processes(), PROC_PAGE_SIZE);
            },
            Event::PrevProcPage => {
                if self.current_page_idx > 0 {
                    self.current_page_idx -= 1;
                }
            },
            Event::NextProcPage => {
                if self.current_page_idx < self.processes.len() - 1 {
                    self.current_page_idx += 1;
                }
            },
            Event::UpdateSelectProc(pid) => {
                self.select_proc = Some(pid);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Event, Theme> {
        let navbar = container(
            column![
                create_nav_button(0, Event::NavTo(Page::Home), self.current_page_button),
                create_nav_button(1, Event::NavTo(Page::Process), self.current_page_button),
                create_nav_button(2, Event::NavTo(Page::Settings), self.current_page_button),
            ]
                .height(Fill)
                .width(Fill)
                .spacing(3)
                .align_x(Alignment::Center)
        )
            .width(Fixed(NAV_BUTTON_WIDTH))
            .style(|_| {
                container::Style::default()
                    .background(Background::Color(color!(18, 18, 18, 0.9)))
            });
        row![
            navbar,
            view(&self.current_page, self)
        ].into()
    }
}
pub fn start(settings: Settings) -> iced::Result {
    iced::application(
        || SysGuard::new(),
        SysGuard::update,
        SysGuard::view
    )
        .title("SysGuard")
        .window(settings)
        .run()
}