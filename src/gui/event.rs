use super::pages::Page;
use super::app::{FilterOptions, ProcessOperation};
#[derive(Debug, Clone)]
pub enum Event {
    NavTo(Page),
    Refresh,
    PrevProcPage,
    NextProcPage,
    UpdateSelectProc(usize),
    UpdateFilteringProcessInput(String),
    FilterProcess,
    UpdateFilterOption(FilterOptions),
    PerformProcessOperations(ProcessOperation),
    PromptCancelled,
    KillProcessAndHidePromptBox,
    WindowResized(u32, u32),
    CopyPromptContent,
}