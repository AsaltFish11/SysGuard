use super::pages::Page;

#[derive(Debug, Clone)]
pub enum Event {
    NavTo(Page),
    Refresh,
    PrevProcPage,
    NextProcPage,
    UpdateSelectProc(usize),
    UpdateFilteringProcessInput(String),
    FilterProcess,
}