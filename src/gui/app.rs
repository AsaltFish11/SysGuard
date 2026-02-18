use super::super::constant::*;
use super::super::utils::handle::*;
use super::super::utils::process::*;
use super::event::Event;
use super::pages::{
    view, Page
};
use iced::border::Radius;
use iced::widget::{
    button, column, container,
    row, scrollable, text,
    Button
};
use iced::Length::Fixed;
use iced::{color, Alignment, Background, Element, Fill, Subscription, Task, Theme};
use iced::{window, Padding};
use iced_dialog::dialog;
use std::fmt::Write;
use sysinfo::{Pid, ProcessesToUpdate, System};
use arboard::Clipboard;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOptions {
    Pid,
    Name,
}
impl std::fmt::Display for FilterOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pid => "进程 ID",
            Self::Name => "进程名",
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOperation {
    DefaultPrompt,
    KillProcess,
    GetProcessDetailedInformation
}
impl std::fmt::Display for ProcessOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DefaultPrompt => "选择要执行的操作",
            Self::KillProcess => "结束选中进程",
            Self::GetProcessDetailedInformation => "获取进程详细信息"
        })
    }
}

pub struct OptionsEntry {
    name: String,
    event: Event
}
pub struct OptionsEntryList {
    options: Vec<OptionsEntry>,
    default_event: Event
}
pub struct SysGuard {
    window_width: u32,
    window_height: u32,
    current_page: Page,
    current_page_button: usize,
    pub processes: Vec<Vec<ProcessEntry>>,
    pub processes_v: Vec<ProcessEntry>,
    pub current_page_idx: usize,
    pub select_proc: Option<usize>,
    pub filtering_process_input_content: String,
    pub filter_option: FilterOptions,
    pub prompt_box_content_str: String,
    pub prompt_box_title: String,
    pub options_list: OptionsEntryList,
    pub prompt_box_is_open: bool,
    pub prompt_size: (u32, u32),
    pub prompt_additional_component: Option<Vec<PromptComponent>>,
    pub clipboard: Option<Clipboard>,
}
#[derive(Clone, Debug)]
pub enum PromptComponent {
    Text(String),
    Button { label: String, on_press: Event },
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
fn filter_by_option(
    processes_v: &Vec<ProcessEntry>,
    filter_option: FilterOptions,
    filtering_process_input_content: &str
) -> Vec<Vec<ProcessEntry>>{
    let mut tmp_proc_vec = Vec::new();
    for process in processes_v.iter() {
        match filter_option {
            FilterOptions::Pid => {
                if process.pid.to_string().contains(filtering_process_input_content) {
                    tmp_proc_vec.push(process.clone());
                }
            },
            FilterOptions::Name => {
                if process.name.contains(filtering_process_input_content) {
                    tmp_proc_vec.push(process.clone());
                }
            }
        }
    }
    list_pagination(tmp_proc_vec, PROC_PAGE_SIZE)
}
impl SysGuard {
    pub fn new() -> Self {

        let clipboard: Option<Clipboard>;
        if let Ok(clipboard_c) = Clipboard::new() {
            clipboard = Some(clipboard_c);
        }else {
            clipboard = None;
        }
        SysGuard {
            window_width: 0,
            window_height: 0,
            current_page: Page::Home,
            current_page_button: 0,
            processes_v: fetch_processes(),
            processes: list_pagination(fetch_processes(), PROC_PAGE_SIZE),
            current_page_idx: 0,
            select_proc: None,
            filtering_process_input_content: String::new(),
            filter_option: FilterOptions::Name,
            prompt_box_content_str: String::new(),
            options_list: OptionsEntryList {
                options: Vec::new(),
                default_event: Event::PromptCancelled
            },
            prompt_box_is_open: false,
            prompt_box_title: String::new(),
            prompt_size: (500, 400),
            prompt_additional_component: None,
            clipboard,
        }
    }
    pub fn create_prompt_box(
        &mut self,
        title: String,
        content: String,
        options_name: Vec<String>,
        options_event: Vec<Event>,
        default_event: Event,
        prompt_component: Option<Vec<PromptComponent>>
    ) {
        if options_name.len() != options_event.len() {
            panic!("名称和事件数量不匹配");
        }
        self.prompt_box_title = title;
        self.prompt_box_content_str = content;
        let mut options_list = Vec::new();
        for i in 0..options_name.len() {
            options_list.push(OptionsEntry {
                name: options_name[i].clone(),
                event: options_event[i].clone()
            });
        }
        self.options_list = OptionsEntryList {
            options: options_list,
            default_event
        };
        self.prompt_box_is_open = true;
        self.prompt_additional_component = prompt_component;
    }
    pub fn update(&mut self, msg: Event) -> Task<Event> {
        match msg {
            Event::NavTo(page) => {
                self.current_page = page;
                self.current_page_button = page.page_idx();
                Task::none()
            },
            Event::Refresh => {
                self.processes_v = fetch_processes();
                self.current_page_idx = 0;
                self.processes = filter_by_option(
                    &self.processes_v,
                    self.filter_option,
                    &self.filtering_process_input_content
                );
                Task::none()
            },
            Event::PrevProcPage => {
                if self.current_page_idx > 0 {
                    self.current_page_idx -= 1;
                }
                Task::none()
            },
            Event::NextProcPage => {
                if self.current_page_idx < self.processes.len() - 1 {
                    self.current_page_idx += 1;
                }
                Task::none()
            },
            Event::UpdateSelectProc(pid) => {
                if self.select_proc == Some(pid) {
                    self.select_proc = None;
                    return Task::none();
                }
                self.select_proc = Some(pid);
                Task::none()
            },
            Event::UpdateFilteringProcessInput(content) => {
                self.filtering_process_input_content = content;
                Task::none()
            }
            Event::FilterProcess => {
                self.current_page_idx = 0;
                self.processes = filter_by_option(
                    &self.processes_v,
                    self.filter_option,
                    &self.filtering_process_input_content
                );
                Task::none()
            },
            Event::UpdateFilterOption(option) => {
                self.filter_option = option;
                Task::none()
            },
            Event::PerformProcessOperations(operations) => {
                if let Some(proc_idx) = self.select_proc {
                    let u_select = &self.processes[self.current_page_idx][proc_idx];
                    let u_select_pid = u_select.pid;
                    let mut system = System::new_all();
                    // 刷新所有进程
                    system.refresh_processes(ProcessesToUpdate::All, true);
                    match operations {
                        ProcessOperation::DefaultPrompt => {},
                        ProcessOperation::KillProcess => {
                            if let Some(_) = system.process(Pid::from_u32(u_select_pid)) {
                                self.create_prompt_box(
                                    "提示".to_string(),
                                    format!("是否结束进程: {}", u_select.name).to_string(),
                                    vec!["是".to_string(), "否".to_string()],
                                    vec![Event::KillProcessAndHidePromptBox, Event::PromptCancelled],
                                    Event::PromptCancelled,
                                    None
                                );
                            }else {
                                self.create_prompt_box(
                                    "提示".to_string(),
                                    "进程不存在".to_string(),
                                    vec!["确定".to_string()],
                                    vec![Event::PromptCancelled],
                                    Event::PromptCancelled,
                                    None
                                )
                            }
                        },
                        ProcessOperation::GetProcessDetailedInformation => {
                            if let Some(process) = system.process(Pid::from_u32(u_select_pid)) {
                                let mut process_info = String::new();
                                write!(&mut process_info, "进程 ID: {}\n", process.pid().to_string()).unwrap();
                                write!(&mut process_info, "进程名: {}\n", process.name().to_str().unwrap()).unwrap();
                                let parent_pid = if let Some(parent_pid) = process.parent() {
                                    parent_pid.to_string()
                                } else {
                                    "无".to_string()
                                };
                                write!(&mut process_info, "父进程ID: {}\n", parent_pid).unwrap();
                                let exe_path = if let Some(exe_path) = process.exe() {
                                    exe_path.to_string_lossy().to_string()
                                } else {
                                    "未知（无权限读取）".to_string()
                                };
                                write!(&mut process_info, "可执行文件路径: {}\n", exe_path).unwrap();
                                let cmd_line_args = process.cmd()
                                    .iter().map(|x| {x.to_string_lossy().to_string()})
                                    .collect::<Vec<_>>().join(", ");
                                write!(&mut process_info, "命令行参数: {}\n", cmd_line_args).unwrap();
                                write!(&mut process_info, "进程状态: {}\n", process.status()).unwrap();
                                write!(&mut process_info, "运行时间: {}\n", process.run_time()).unwrap();
                                let disk_usage = process.disk_usage();
                                write!(&mut process_info, "总读取字节: {}\n", disk_usage.total_read_bytes).unwrap();
                                write!(&mut process_info, "总写入字节: {}\n", disk_usage.total_written_bytes).unwrap();
                                let environ_str = process.environ().iter()
                                    .map(|x| {x.to_string_lossy().to_string()}).collect::<Vec<_>>()
                                    .join(", ");
                                write!(&mut process_info, "环境变量: {}\n", environ_str).unwrap();
                                if let Some(cwd) = process.cwd() {
                                    write!(&mut process_info, "当前进程工作目录: {}\n", cwd.to_string_lossy().to_string()).unwrap();
                                }else {
                                    write!(&mut process_info, "当前进程工作目录: 无\n").unwrap();
                                }
                                write!(&mut process_info, "启动时间: {}\n", process.start_time()).unwrap();
                                write!(&mut process_info, "已运行时间: {}\n", process.run_time()).unwrap();
                                write!(&mut process_info, "累计的 CPU 使用时间: {}\n", process.accumulated_cpu_time()).unwrap();
                                let user_id = if let Some(user_id) = process.user_id() {
                                    user_id.to_string()
                                } else {
                                    "无法获取".to_string()
                                };
                                write!(&mut process_info, "所有者 UID: {}\n", user_id).unwrap();
                                let effective_user_id = if let Some(effective_user_id) = process.effective_user_id() {
                                    effective_user_id.to_string()
                                } else {
                                    "无法获取".to_string()
                                };
                                write!(&mut process_info, "有效所有者 UID: {}\n", effective_user_id).unwrap();
                                if let Some(group_id) = process.group_id() {
                                    write!(&mut process_info, "所有者 GID: {}\n", group_id.to_string()).unwrap();
                                }
                                if let Some(effective_group_id) = process.effective_group_id() {
                                    write!(&mut process_info, "进程的有效组 ID: {}\n", effective_group_id.to_string()).unwrap();
                                }
                                if let Some(session_id) = process.session_id() {
                                    write!(&mut process_info, "进程会话 ID: {}\n", session_id.to_string()).unwrap();
                                }else {
                                    write!(&mut process_info, "进程会话 ID: 无法获取\n").unwrap();
                                }
                                if let Some(tasks) = process.tasks() {
                                    write!(&mut process_info, "此进程运行的的任务 Pid: {}", tasks.iter().map(|x1| {x1.to_string()}).collect::<Vec<_>>().join(", ")).unwrap();
                                    write!(&mut process_info, "总计: {}个\n", tasks.len()).unwrap();
                                }
                                if let Some(open_files_num) = process.open_files() {
                                    write!(&mut process_info, "进程打开的文件数: {}\n", open_files_num).unwrap();
                                }
                                if let Some(open_files_limit) = process.open_files_limit() {
                                    write!(&mut process_info, "当前进程的最大打开文件数限制: {}\n", open_files_limit).unwrap();
                                }

                                self.create_prompt_box(
                                    "进程详细信息".to_string(),
                                    process_info,
                                    vec!["确定".to_string()],
                                    vec![Event::PromptCancelled],
                                    Event::PromptCancelled,
                                    Some(vec!(
                                        PromptComponent::Button {
                                            label: "复制信息".to_string(),
                                            on_press: Event::CopyPromptContent
                                        }
                                    ))
                                )
                            }
                        }
                    }
                }
                else {
                    self.create_prompt_box(
                        "提示".to_string(),
                        "请选择进程".to_string(),
                        vec!["确定".to_string()],
                        vec![Event::PromptCancelled],
                        Event::PromptCancelled,
                        None
                    )
                }
                Task::none()
            },
            Event::PromptCancelled => {
                self.prompt_box_is_open = false;
                Task::none()
            },
            Event::KillProcessAndHidePromptBox => {
                self.prompt_box_is_open = false;
                if let Some(proc_idx) = self.select_proc {
                    let u_select_pif = self.processes[self.current_page_idx][proc_idx].pid;
                    let mut system = System::new_all();
                    // 刷新所有进程
                    system.refresh_processes(ProcessesToUpdate::All, true);
                    if let Some(process) = system.process(Pid::from_u32(u_select_pif)) {
                        process.kill();
                    }
                }
                Task::none()
            },
            Event::WindowResized(width, height) => {
                (self.window_width, self.window_height) = (width, height);
                Task::none()
            },
            Event::CopyPromptContent => {
                if self.clipboard.is_none() {
                    self.clipboard = match Clipboard::new() {
                        Ok(clipboard) => Some(clipboard),
                        Err(e) => {
                            println!("无法创建剪贴板: {:?}", e);
                            return Task::none();
                        }
                    }
                }
                match self.clipboard {
                    Some(ref mut clipboard) => {
                        if let Err(e) = clipboard.set_text(self.prompt_box_content_str.clone()) {
                            println!("无法复制内容到剪贴板: {:?}", e);
                        }
                    },
                    None => {}
                }
                Task::none()
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
        let main_content = row![
            navbar,
            view(&self.current_page, self)
        ];
        let mut prompt_main_component =
            column![
                container(text(&self.prompt_box_content_str)),
            ];
        if let Some(prompt_component) = &self.prompt_additional_component {
            for component in prompt_component {
                match component {
                    PromptComponent::Text(text_str) => {
                        prompt_main_component = prompt_main_component.push(text(text_str));
                    },
                    PromptComponent::Button { label, on_press } => {
                        prompt_main_component = prompt_main_component.push(
                            button(text(label))
                                .on_press(on_press.clone())
                        );
                    }
                }
            }
        }
        let prompt_component = scrollable(
            prompt_main_component.padding(Padding::from(0).right(15).bottom(15))
        ).width(Fill).height(240).direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::new().spacing(10),   // 启用垂直滚动条
            horizontal: scrollable::Scrollbar::new().spacing(10), // 启用水平滚动条
        });
        let mut prompt_box = dialog(self.prompt_box_is_open,
                                    main_content,
                                    prompt_component)
            .title(&self.prompt_box_title)
            .width(self.prompt_size.0)
            .height(self.prompt_size.1)
            .on_press(self.options_list.default_event.clone());
        for dialog_entry in self.options_list.options.iter() {
            prompt_box = prompt_box.push_button(
                iced_dialog::button(dialog_entry.name.clone(), dialog_entry.event.clone())
            );
        }
        prompt_box.into()
    }

    fn subscription(&self) -> Subscription<Event> {
        window::events()
            .map(|(_, event)| match event {
                window::Event::Resized(size) => {
                    Some(Event::WindowResized(size.width as u32, size.height as u32))
                }
                _ => None
            })
            .filter_map(|x| x)
    }

}
pub fn start(settings: window::Settings) -> iced::Result {
    iced::application(
        || SysGuard::new(),
        SysGuard::update,
        SysGuard::view
    )
        .title("SysGuard")
        .window(settings)
        .theme(Theme::Dark)
        .subscription(SysGuard::subscription)
        .run()
}