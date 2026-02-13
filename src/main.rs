use iced::border::Radius;
use iced::widget::button::Status;
use iced::widget::{
    button, column, container,
    row, text, rule, Button,
    scrollable
};
use iced::window::Settings;
use iced::Length::Fixed;
use iced::{
    color,
    Alignment, Background, Element,
    Fill, FillPortion, Size,
    Theme
};
use sysinfo::{ProcessesToUpdate, System};

const NAV_BUTTON_TEXT: [&str; 3] = ["首页", "设置", "测试页1"];
const NAV_BUTTON_WIDTH: f32 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    Home,
    Settings,
    Test1,
}

impl Page {
    fn page_idx(&self) -> usize {
        match self {
            Page::Home => 0,
            Page::Settings => 1,
            Page::Test1 => 2,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    NavTo(Page),
    Refresh,
}

#[derive(Debug, Clone)]
struct ProcessEntry {
    pid: u32,
    name: String,
    cpu: f32,
    memory: u64,
}

struct SysGuard {
    current_page: Page,
    current_page_button: usize,
    processes: Vec<ProcessEntry>,
}

fn fetch_processes() -> Vec<ProcessEntry> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    sys.processes()
        .iter()
        .map(|(pid, process)| ProcessEntry {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().into_owned(),
            cpu: process.cpu_usage(),
            memory: process.memory(),
        })
        .collect()
}

fn create_nav_button(
    nav_button_idx: usize,
    on_press: Message,
    current_page_idx: usize,
) -> Button<'static, Message, Theme> {
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
                Status::Hovered => hover_bg,
                Status::Pressed => pressed_bg,
                _ => normal_bg,
            }));

            style.border.color = color!(61, 174, 233);
            style.border.width = if status == Status::Hovered { 1.0 } else { 0.0 };
            style.border.radius = Radius::new(4);
            style
        })
}

impl SysGuard {
    fn new() -> Self {
        Self {
            current_page: Page::Home,
            current_page_button: 0,
            processes: Vec::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NavTo(page) => {
                self.current_page = page;
                self.current_page_button = self.current_page.page_idx();
                if page == Page::Home {
                    self.processes = fetch_processes();
                }
            }
            Message::Refresh => {
                self.processes = fetch_processes();
            }
        }
    }

    fn view(&self) -> Element<'_, Message, Theme> {
        // 侧边导航栏
        let navbar = container(
            column![
                create_nav_button(0, Message::NavTo(Page::Home), self.current_page_button),
                create_nav_button(1, Message::NavTo(Page::Settings), self.current_page_button),
                create_nav_button(2, Message::NavTo(Page::Test1), self.current_page_button),
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

        // 页面内容逻辑
        let content: Element<'_, Message, Theme> = match self.current_page {
            Page::Home => {
                let process_list: Vec<Element<'_, Message, Theme>> = self.processes
                    .iter()
                    .map(|p| {
                        row![
                            text(format!("{}", p.pid)).width(80),
                            text(&p.name).width(200),
                            text(format!("{:.1}%", p.cpu)).width(80),
                            text(format!("{} KB", p.memory / 1024)).width(100),
                        ].into()
                    })
                    .collect();

                column![
                    button("刷新").on_press(Message::Refresh),
                    scrollable(column(process_list).spacing(10))
                ]
                    .into()
            }
            Page::Settings => {
                column![
                    text("这是设置页面").size(30),
                    text("这是设置页面重复1").size(30)
                ]
                    .into()
            }
            Page::Test1 => {
                column![
                    text("这是测试页面").size(30),
                    text("这是测试页面重复1").size(30),
                ]
                    .into()
            }
        };

        let page_body = container(content)
            .width(FillPortion(5))
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .style(|_| {
                container::Style::default()
                    .background(Background::Color(color!(18, 18, 18, 0.9)))
            });

        row![
            navbar,
            rule::vertical(1),
            page_body
        ]
            .into()
    }
}

fn main() -> iced::Result {
    let mut app_window_setting = Settings::default();
    app_window_setting.min_size = Some(Size::new(800.0, 600.0));

    iced::application(
        || SysGuard::new(),
        SysGuard::update,
        SysGuard::view
    )
        .theme(Theme::Dark)
        .title("SysGuard")
        .window(app_window_setting)
        .run()
}