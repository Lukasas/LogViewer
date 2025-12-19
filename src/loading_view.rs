use iced::{Element, widget::{text, container, progress_bar}};

use crate::{LogReader, log_message::LogReaderMessage};



pub fn view(log_reader: &LogReader) -> Element<'_, LogReaderMessage> {
    // progress_bar(0..=100, value)

    container(text("Hello")).into()
}

pub fn update(log_reader: &mut LogReader, msg: LogReaderMessage) {

}