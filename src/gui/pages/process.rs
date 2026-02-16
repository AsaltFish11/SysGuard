use iced::{color, Alignment, Background, Element, Fill, Theme};
use iced::widget::{container, mouse_area, row, text, column, scrollable, button};
use super::super::app::SysGuard;
use super::super::message::Event;

pub fn view(sys_guard: &SysGuard) -> Element<'_, Event, Theme> {
    let proc_list:  Vec<Element<'_, Event, Theme>> = sys_guard.processes[sys_guard.current_page_idx]
        .iter().enumerate()
        .map(move |(index, process)| {
            let is_selected = sys_guard.select_proc == Some(index);
            let row_content = row![
                text(process.pid.to_string()).width(80),
                text(&process.name).width(200)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .padding(5);

            container(mouse_area(row_content)
                .on_press(Event::UpdateSelectProc(index)))
                .width(Fill)
                .style(move |_| {
                    if is_selected {
                        container::Style {
                            background: Some(Background::Color(color!(71, 194, 253))),
                            ..container::Style::default()
                        }
                    } else {
                        container::Style::default()
                    }
                })
                .into()
        }).collect();
        column![
            scrollable(
                column(proc_list)
            ).width(Fill).height(Fill),
            row![
                button("刷新").on_press(Event::Refresh),
                button("上一页").on_press(Event::PrevProcPage),
                button("下一页").on_press(Event::NextProcPage),
                text!("第 {} 页，共 {} 页", sys_guard.current_page_idx + 1, sys_guard.processes.len())
            ].width(Fill),
        ]
            .align_x(Alignment::Center)
            .into()
}