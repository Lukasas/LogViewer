use iced::{Event as IcedEvent, widget::text_editor, window};

#[derive(Debug, Clone)]
pub enum FilterMessage {}

#[derive(Debug, Clone)]
pub enum LogReaderMessage {
    // Change(usize),
    ScrollChanged(f64),
    Event(IcedEvent),
    WindowClosed(window::Id),

    TableEntered,
    TableLeft,

    SearchInputChanged(String),

    TextEditorEdit(text_editor::Action),

    IDInput(String),
    DateInput(String),
    ComputerNameInput(String),
    ProcessIDInput(String),
    ProcessUserInput(String),
    ModuleNameInput(String),
    MessageIDInput(String),
    LogLevelInput(String),
    FacilityInput(String),
    LogMessageInput(String),
}
