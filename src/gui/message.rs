use super::pages::Page;
use super::app::FilterOptions;

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
}