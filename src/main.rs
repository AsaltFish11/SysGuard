use iced::border::Radius;
use iced::widget::button::Status;
use iced::widget::{
    button, column, container,
    row, text, rule, Button,
    scrollable, text_input, Space
};
use iced::window::Settings;
use iced::Length::Fixed;
use iced::{
    color,
    Alignment, Background, Border, Element,
    Fill, FillPortion, Size, Task,
    Theme
};
use sysinfo::{ProcessesToUpdate, System};
use std::time::Duration;

const NAV_BUTTON_TEXT: [&str; 4] = ["首页", "进程管理", "设置", "测试页1"];
const NAV_BUTTON_WIDTH: f32 = 200.0;
const ITEMS_PER_PAGE: usize = 50; // 每页显示的进程数
const REFRESH_INTERVAL: u64 = 2; // 刷新间隔（秒）

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    Home,
    ProcessManager,
    Settings,
    Test1
}

impl Page {
    fn page_idx(&self) -> usize {
        match self {
            Page::Home => 0,
            Page::ProcessManager => 1,
            Page::Settings => 2,
            Page::Test1 => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    fn symbol(&self) -> &str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    NavTo(Page),
    Refresh,
    SortBy(SortColumn),
    SearchChanged(String),
    NextPage,
    PrevPage,
    Tick, // 定时刷新
}

struct SysGuard {
    current_page: Page,
    current_page_button: usize,
    processes: Vec<ProcessEntry>,
    filtered_processes: Vec<ProcessEntry>,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    search_query: String,
    current_display_page: usize,
    self_pid: u32, // 自身进程PID
}

fn create_nav_button(
    nav_button_idx: usize,
    on_press: Message,
    current_page_idx: usize,
) -> Button<'static, Message, Theme> {
    let is_select = nav_button_idx == current_page_idx;
    button(text(NAV_BUTTON_TEXT[nav_button_idx]).size(16))
        .on_press(on_press)
        .width(Fixed(NAV_BUTTON_WIDTH-8.0))
        .padding([12, 16])
        .style(move |theme: &Theme, status| {
            let mut style = button::primary(theme, status);
            let (normal_bg, hover_bg, pressed_bg) = if is_select {
                (
                    color!(0, 122, 204),
                    color!(0, 142, 224),
                    color!(0, 102, 184),
                )
            } else {
                (
                    color!(28, 28, 28),
                    color!(40, 40, 40),
                    color!(50, 50, 50)
                )
            };

            style.background = Some(Background::Color(match status {
                Status::Hovered => hover_bg,
                Status::Pressed => pressed_bg,
                _ => normal_bg,
            }));
            style.border.color = if is_select {
                color!(0, 122, 204)
            } else {
                color!(45, 45, 45)
            };
            style.border.width = 1.0;
            style.border.radius = 6.0.into();
            style
        })
}

#[derive(Debug, Clone)]
struct ProcessEntry {
    pid: u32,
    name: String,
    cpu: f32,
    memory: u64,
}

fn fetch_processes(exclude_pid: u32) -> Vec<ProcessEntry> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    sys.processes()
        .iter()
        .filter_map(|(pid, process)| {
            let pid_u32 = pid.as_u32();
            // 排除自身进程
            if pid_u32 == exclude_pid {
                None
            } else {
                Some(ProcessEntry {
                    pid: pid_u32,
                    name: process.name().to_string_lossy().into_owned(),
                    cpu: process.cpu_usage(),
                    memory: process.memory(),
                })
            }
        })
        .collect()
}

impl SysGuard {
    fn new() -> (Self, Task<Message>) {
        let self_pid = std::process::id();
        let mut instance = Self {
            current_page: Page::Home,
            current_page_button: 0,
            processes: Vec::new(),
            filtered_processes: Vec::new(),
            sort_column: SortColumn::Cpu,
            sort_direction: SortDirection::Descending,
            search_query: String::new(),
            current_display_page: 0,
            self_pid,
        };
        instance.processes = fetch_processes(self_pid);
        instance.apply_filter();
        instance.sort_processes();

        // 启动定时器
        let task = Task::run(
            async {
                tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL)).await;
                Message::Tick
            }
        );

        (instance, task)
    }

    fn create_header_button(
        &self,
        label: &str,
        column: SortColumn,
        width_portion: u16,
    ) -> Button<'_, Message, Theme> {
        let is_active = self.sort_column == column;
        let display_text = if is_active {
            format!("{} {}", label, self.sort_direction.symbol())
        } else {
            label.to_string()
        };

        button(text(display_text).size(14))
            .on_press(Message::SortBy(column))
            .width(FillPortion(width_portion))
            .padding([10, 8])
            .style(move |theme: &Theme, status| {
                let mut style = button::primary(theme, status);
                let bg_color = if is_active {
                    color!(0, 102, 184)
                } else {
                    match status {
                        Status::Hovered => color!(45, 45, 45),
                        Status::Pressed => color!(55, 55, 55),
                        _ => color!(35, 35, 35),
                    }
                };

                style.background = Some(Background::Color(bg_color));
                style.border = Border {
                    color: color!(60, 60, 60),
                    width: 1.0,
                    radius: 0.0.into(),
                };
                style.text_color = color!(220, 220, 220);
                style
            })
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_processes = self.processes.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_processes = self.processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query)
                        || p.pid.to_string().contains(&query)
                })
                .cloned()
                .collect();
        }
        self.current_display_page = 0;
    }

    fn sort_processes(&mut self) {
        match self.sort_column {
            SortColumn::Pid => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.pid.cmp(&b.pid);
                    if self.sort_direction == SortDirection::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortColumn::Name => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                    if self.sort_direction == SortDirection::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortColumn::Cpu => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.cpu.partial_cmp(&b.cpu).unwrap_or(std::cmp::Ordering::Equal);
                    if self.sort_direction == SortDirection::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortColumn::Memory => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.memory.cmp(&b.memory);
                    if self.sort_direction == SortDirection::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
        }
    }

    fn total_pages(&self) -> usize {
        (self.filtered_processes.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE
    }

    fn view_process_manager(&self) -> Element<'_, Message, Theme> {
        // 工具栏
        let toolbar = container(
            row![
                text_input("搜索进程名或PID...", &self.search_query)
                    .on_input(Message::SearchChanged)
                    .padding(10)
                    .width(300),
                button(text("🔄 刷新").size(14))
                    .on_press(Message::Refresh)
                    .padding([10, 20])
                    .style(|theme: &Theme, status| {
                        let mut style = button::primary(theme, status);
                        style.background = Some(Background::Color(match status {
                            Status::Hovered => color!(0, 142, 224),
                            Status::Pressed => color!(0, 102, 184),
                            _ => color!(0, 122, 204),
                        }));
                        style.border.radius = 6.0.into();
                        style
                    }),
                text(format!("总计: {} 进程", self.filtered_processes.len()))
                    .size(14)
                    .color(color!(180, 180, 180)),
            ]
                .spacing(15)
                .align_y(Alignment::Center)
        )
            .padding(15)
            .style(|_theme: &Theme| {
                container::Style::default()
                    .background(Background::Color(color!(30, 30, 30)))
                    .border(Border {
                        color: color!(50, 50, 50),
                        width: 1.0,
                        radius: 8.0.into(),
                    })
            });

        // 表头
        let header = container(
            row![
                self.create_header_button("PID", SortColumn::Pid, 2),
                self.create_header_button("进程名", SortColumn::Name, 5),
                self.create_header_button("CPU使用率", SortColumn::Cpu, 2),
                self.create_header_button("内存", SortColumn::Memory, 3),
            ]
                .spacing(0)
        )
            .style(|_theme: &Theme| {
                container::Style::default()
                    .background(Background::Color(color!(35, 35, 35)))
                    .border(Border {
                        color: color!(60, 60, 60),
                        width: 1.0,
                        radius: Radius::new(8.0),
                    })
            });

        // 分页处理
        let start_idx = self.current_display_page * ITEMS_PER_PAGE;
        let end_idx = (start_idx + ITEMS_PER_PAGE).min(self.filtered_processes.len());
        let page_processes = &self.filtered_processes[start_idx..end_idx];

        // 进程列表
        let process_rows: Vec<Element<'_, Message, Theme>> = page_processes
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let bg_color = if idx % 2 == 0 {
                    color!(28, 28, 28)
                } else {
                    color!(32, 32, 32)
                };

                let cpu_color = if p.cpu > 50.0 {
                    color!(255, 100, 100)
                } else if p.cpu > 20.0 {
                    color!(255, 200, 100)
                } else {
                    color!(200, 200, 200)
                };

                let mem_mb = p.memory as f64 / 1024.0 / 1024.0;
                let mem_text = if mem_mb > 1024.0 {
                    format!("{:.2} GB", mem_mb / 1024.0)
                } else {
                    format!("{:.1} MB", mem_mb)
                };

                container(
                    row![
                        text(format!("{}", p.pid))
                            .width(FillPortion(2))
                            .size(13),
                        text(&p.name)
                            .width(FillPortion(5))
                            .size(13),
                        text(format!("{:.1}%", p.cpu))
                            .width(FillPortion(2))
                            .size(13)
                            .color(cpu_color),
                        text(mem_text)
                            .width(FillPortion(3))
                            .size(13),
                    ]
                        .padding([8, 10])
                        .spacing(0)
                )
                    .style(move |_theme: &Theme| {
                        container::Style::default()
                            .background(Background::Color(bg_color))
                            .border(Border {
                                color: color!(45, 45, 45),
                                width: 0.5,
                                radius: 0.0.into(),
                            })
                    })
                    .into()
            })
            .collect();

        let process_list = container(
            scrollable(
                column(process_rows).spacing(0)
            )
                .height(Fill)
        )
            .height(Fill)
            .style(|_theme: &Theme| {
                container::Style::default()
                    .background(Background::Color(color!(28, 28, 28)))
                    .border(Border {
                        color: color!(60, 60, 60),
                        width: 1.0,
                        radius: Radius::new(8.0),
                    })
            });

        // 分页控制
        let total_pages = self.total_pages();
        let pagination_button_style = |_theme: &Theme, status| {
            let mut style = button::primary(_theme, status);
            style.background = Some(Background::Color(
                if status == Status::Hovered {
                    color!(45, 45, 45)
                } else {
                    color!(35, 35, 35)
                }
            ));
            style.border.radius = 6.0.into();
            style
        };

        let pagination = if total_pages > 1 {
            container(
                row![
                    button(text("◀ 上一页").size(13))
                        .on_press_maybe(
                            if self.current_display_page > 0 {
                                Some(Message::PrevPage)
                            } else {
                                None
                            }
                        )
                        .padding([8, 16])
                        .style(pagination_button_style),
                    text(format!(
                        "第 {} / {} 页 (显示 {}-{} / {} 项)",
                        self.current_display_page + 1,
                        total_pages,
                        start_idx + 1,
                        end_idx,
                        self.filtered_processes.len()
                    ))
                    .size(13)
                    .color(color!(180, 180, 180)),
                    button(text("下一页 ▶").size(13))
                        .on_press_maybe(
                            if self.current_display_page + 1 < total_pages {
                                Some(Message::NextPage)
                            } else {
                                None
                            }
                        )
                        .padding([8, 16])
                        .style(pagination_button_style),
                ]
                    .spacing(20)
                    .align_y(Alignment::Center)
            )
                .padding(15)
                .style(|_theme: &Theme| {
                    container::Style::default()
                        .background(Background::Color(color!(30, 30, 30)))
                        .border(Border {
                            color: color!(50, 50, 50),
                            width: 1.0,
                            radius: 8.0.into(),
                        })
                })
        } else {
            container(Space::new())
        };

        column![
            toolbar,
            header,
            process_list,
            pagination,
        ]
            .spacing(10)
            .padding(20)
            .into()
    }

    fn view(&self) -> Element<'_, Message, Theme> {
        let navbar = container(
            column![
                create_nav_button(0, Message::NavTo(Page::Home), self.current_page_button),
                create_nav_button(1, Message::NavTo(Page::ProcessManager), self.current_page_button),
                create_nav_button(2, Message::NavTo(Page::Settings), self.current_page_button),
                create_nav_button(3, Message::NavTo(Page::Test1), self.current_page_button),
            ]
                .height(Fill)
                .width(Fill)
                .spacing(8)
                .padding([20, 0])
                .align_x(Alignment::Center)
        )
            .width(Fixed(NAV_BUTTON_WIDTH))
            .height(Fill)
            .padding(10)
            .style(|_theme: &Theme| {
                container::Style::default()
                    .background(Background::Color(color!(24, 24, 24)))
                    .border(Border {
                        color: color!(45, 45, 45),
                        width: 1.0,
                        radius: 0.0.into(),
                    })
            });

        let content: Element<'_, Message, Theme> = match self.current_page {
            Page::Home => {
                container(
                    text("🏠 SysGuard").size(48).color(color!(200, 200, 200))
                )
                    .center_x(Fill)
                    .center_y(Fill)
                    .into()
            },
            Page::ProcessManager => self.view_process_manager(),
            Page::Settings => {
                container(
                    column![
                        text("⚙️ 设置页面").size(32).color(color!(200, 200, 200)),
                        rule::horizontal(20),
                        text("这里可以添加各种设置选项").size(16).color(color!(150, 150, 150)),
                    ]
                        .spacing(20)
                        .padding(40)
                )
                    .center_x(Fill)
                    .center_y(Fill)
                    .into()
            },
            Page::Test1 => {
                container(
                    column![
                        text("📋 测试页面").size(32).color(color!(200, 200, 200)),
                        rule::horizontal(20),
                        text("这里可以添加测试功能").size(16).color(color!(150, 150, 150)),
                    ]
                        .spacing(20)
                        .padding(40)
                )
                    .center_x(Fill)
                    .center_y(Fill)
                    .into()
            }
        };

        let page_body = container(content)
            .width(FillPortion(5))
            .height(Fill)
            .style(|_theme: &Theme| {
                container::Style::default()
                    .background(Background::Color(color!(20, 20, 20)))
            });

        row![
            navbar,
            page_body
        ].into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavTo(page) => {
                self.current_page = page;
                self.current_page_button = self.current_page.page_idx();
                if page == Page::ProcessManager && self.processes.is_empty() {
                    self.processes = fetch_processes(self.self_pid);
                    self.apply_filter();
                    self.sort_processes();
                }
                Task::none()
            },
            Message::Refresh => {
                self.processes = fetch_processes(self.self_pid);
                self.apply_filter();
                self.sort_processes();
                Task::none()
            },
            Message::SortBy(column) => {
                if self.sort_column == column {
                    self.sort_direction = self.sort_direction.toggle();
                } else {
                    self.sort_column = column;
                    self.sort_direction = SortDirection::Descending;
                }
                self.sort_processes();
                Task::none()
            },
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.apply_filter();
                self.sort_processes();
                Task::none()
            },
            Message::NextPage => {
                if self.current_display_page + 1 < self.total_pages() {
                    self.current_display_page += 1;
                }
                Task::none()
            },
            Message::PrevPage => {
                if self.current_display_page > 0 {
                    self.current_display_page -= 1;
                }
                Task::none()
            },
            Message::Tick => {
                // 只在进程管理页面时自动刷新
                if self.current_page == Page::ProcessManager {
                    self.processes = fetch_processes(self.self_pid);
                    self.apply_filter();
                    self.sort_processes();
                }

                // 启动下一次定时器
                Task::run(
                    async {
                        tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL)).await;
                        Message::Tick
                    }
                )
            },
        }
    }
}

fn main() -> iced::Result {
    let mut app_window_setting = Settings::default();
    app_window_setting.min_size = Some(Size::new(900.0, 650.0));

    iced::application("SysGuard - 系统进程监控", SysGuard::update, SysGuard::view)
        .theme(|_| Theme::Dark)
        .window(app_window_setting)
        .run_with(SysGuard::new)
}
