use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    advanced::graphics::core::keyboard,
    widget::{column, container, row, scrollable, text_editor, text_input},
};

use iced::widget::text_input::Status;

use crate::{LogReader, ViewDetail, log_message::LogReaderMessage};

fn field_mod<'a>(title: &'a str, value: &str) -> Element<'a, LogReaderMessage> {
    container(
        column![
            title,
            text_input("", value).style(|theme: &Theme, status: Status| {
                text_input::Style {
                    value: if theme.extended_palette().is_dark {
                        iced::Color::WHITE
                    } else {
                        iced::Color::BLACK
                    },
                    ..text_input::default(theme, status)
                }
            })
        ]
        .spacing(4),
    )
    .into()
}

pub fn view(log_reader: &LogReader) -> Element<'_, LogReaderMessage> {
    // text_input("", &log_reader.view_detail.as_ref().unwrap().record.log_message).into()
    let ViewDetail { record, content } = log_reader.view_detail.as_ref().unwrap();
    let dt = record.date.format("%d/%m/%Y %H:%M:%S%.3f").to_string();

    const FIELD_SPACING_AMOUNT: u32 = 10;

    column![
        row!(
            field_mod("Log ID", &record.id.to_string()),
            field_mod("Date", &dt)
        )
        .spacing(FIELD_SPACING_AMOUNT),
        row!(
            field_mod("Computer Name", &record.computer_name),
            column![
                row!(
                    field_mod("Process ID", &record.process_id.to_string()),
                    field_mod("Process User", &record.process_user),
                )
                .spacing(FIELD_SPACING_AMOUNT)
            ]
            .spacing(FIELD_SPACING_AMOUNT)
        )
        .spacing(FIELD_SPACING_AMOUNT),
        row!(
            field_mod("Module Name", &record.module_name),
            column![
                row!(
                    field_mod("Message ID", &record.message_id.to_string()),
                    field_mod("Log Level", &record.log_level.to_string()),
                )
                .spacing(FIELD_SPACING_AMOUNT)
            ]
            .spacing(FIELD_SPACING_AMOUNT)
        )
        .spacing(FIELD_SPACING_AMOUNT),
        field_mod("Facility", &record.facility.to_string()),
        container(scrollable(
            text_editor(&content).on_action(LogReaderMessage::TextEditorEdit)
        ))
        .width(Fill)
        .height(Fill)
    ]
    .padding(20)
    .spacing(10)
    .into()
}

pub fn update(log_reader: &mut LogReader, msg: LogReaderMessage) -> Task<LogReaderMessage> {
    match msg {
        LogReaderMessage::TextEditorEdit(action) => match action {
            text_editor::Action::Edit(_) => {}
            _ => {
                log_reader
                    .view_detail
                    .as_mut()
                    .unwrap()
                    .content
                    .perform(action);
            }
        },
        LogReaderMessage::Event(event) => match event {
            iced::Event::Keyboard(keyboard_event) => match keyboard_event {
                iced::keyboard::Event::KeyPressed { key, .. } => {
                    if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                        log_reader.view_detail = None;
                    }
                }
                _ => (),
            },
            _ => (),
        },

        _ => (),
    }

    Task::none()
}
