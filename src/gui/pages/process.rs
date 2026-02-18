use iced::{color, Alignment, Background, Element, Fill, Theme};
use iced::widget::{
    container, mouse_area, row,
    text, column, scrollable, button,
    text_input, pick_list
};
use super::super::app::{SysGuard, FilterOptions, ProcessOperation};
use super::super::event::Event;


pub fn view(sys_guard: &SysGuard) -> Element<'_, Event, Theme> {
    let proc_list: Vec<Element<'_, Event, Theme>>;
    if sys_guard.processes.len() == 0 {
        proc_list = vec![
            text("没有匹配到符合条件的进程").size(20).align_x(Alignment::Center).into()
        ];
    }else {
        proc_list = sys_guard.processes[sys_guard.current_page_idx]
            .iter().enumerate()
            .map(move |(index, process)| {
                let is_selected = sys_guard.select_proc == Some(index);
                let row_content = row![
                text(process.pid.to_string()).width(80),
                text(&process.name).width(Fill)
            ]
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
    }

    let filter_options = [
        FilterOptions::Name,
        FilterOptions::Pid,
    ];

    column![
        scrollable(
            column![
                row![
                    pick_list(
                        filter_options,
                        Some(sys_guard.filter_option),
                        Event::UpdateFilterOption
                    ),
                    text_input(
                        format!("从{}筛选进程...", sys_guard.filter_option).as_str(),
                        &sys_guard.filtering_process_input_content,
                    ).on_input(Event::UpdateFilteringProcessInput),
                    button("筛选")
                        .on_press(Event::FilterProcess)
                ].padding(5),
                row![
                    text("Pid").width(80),
                    text("Name").width(200)
                ].padding(5),
                column(proc_list)
            ]
        ).width(Fill).height(Fill),
        row![
            button("刷新").on_press(Event::Refresh),
            button("上一页").on_press(Event::PrevProcPage),
            button("下一页").on_press(Event::NextProcPage),
            text!("第 {} 页，共 {} 页", sys_guard.current_page_idx + 1, sys_guard.processes.len()),
            pick_list(
                [
                    ProcessOperation::KillProcess,
                    ProcessOperation::GetProcessDetailedInformation,
                ],
                Some(ProcessOperation::DefaultPrompt),
                Event::PerformProcessOperations
            ).width(300)
        ].width(Fill),
    ]
        .align_x(Alignment::Center)
        .into()
}