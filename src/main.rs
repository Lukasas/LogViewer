#![cfg_attr(not(test), windows_subsystem = "windows")]
use iced::widget::{column, container, rich_text, row, span, stack, text, text_editor};
use iced::{Element, Event as IcedEvent, Fill, Subscription, Task, Theme, color, event, window};
use iced::{Length, never};

use std::env;

use crate::log_message::LogReaderMessage;
use crate::record::Record;

mod detail_view;
mod loading_view;
mod log_message;
mod record;
mod table_view;

fn main() -> iced::Result {
    iced::application(LogReader::new, LogReader::update, LogReader::view)
        .subscription(LogReader::subscribtion)
        .window_size((1500, 800))
        .title(LogReader::title)
        .run()
}
// reader.seek_relative(4)?;
//

// #[derive(Debug)]
// enum ScreenView {
//     Loading,
//     Table,
//     Detail,
// }

#[derive(Debug, Clone)]
struct ViewDetail {
    record: Record,
    content: text_editor::Content,
}

pub struct LogColumn<'a, 'b, T> {
    pub header: Element<'a, LogReaderMessage>,
    pub view: Box<dyn Fn(T) -> Element<'a, LogReaderMessage> + 'b>,
    pub width: Length,
}

impl<'a, 'b, T> LogColumn<'a, 'b, T> {
    pub fn new<E>(
        header: impl Into<Element<'a, LogReaderMessage>>,
        view: impl Fn(T) -> E + 'b,
        width: impl Into<Length>,
    ) -> LogColumn<'a, 'b, T>
    where
        E: Into<Element<'a, LogReaderMessage>>,
    {
        LogColumn {
            header: header.into(),
            view: Box::new(move |r| view(r).into()),
            width: width.into(),
        }
    }
}

#[derive(Debug)]
pub struct MyFilter {
    value: String,
    message: fn(String) -> LogReaderMessage,
}

#[derive(Debug)]
pub struct MyFilters {
    log_name: MyFilter,
    date: MyFilter,
    computer_name: MyFilter,
    process_id: MyFilter,
    process_user: MyFilter,
    module_name: MyFilter,
    message_id: MyFilter,
    log_level: MyFilter,
    facility: MyFilter,
    log_message: MyFilter,
}

impl Default for MyFilters {
    fn default() -> Self {
        Self {
            log_name: MyFilter {
                value: String::default(),
                message: LogReaderMessage::IDInput,
            },
            date: MyFilter {
                value: String::default(),
                message: LogReaderMessage::DateInput,
            },
            computer_name: MyFilter {
                value: String::default(),
                message: LogReaderMessage::ComputerNameInput,
            },
            process_id: MyFilter {
                value: String::default(),
                message: LogReaderMessage::ProcessIDInput,
            },
            process_user: MyFilter {
                value: String::default(),
                message: LogReaderMessage::ProcessUserInput,
            },
            module_name: MyFilter {
                value: String::default(),
                message: LogReaderMessage::ModuleNameInput,
            },
            message_id: MyFilter {
                value: String::default(),
                message: LogReaderMessage::MessageIDInput,
            },
            log_level: MyFilter {
                value: String::default(),
                message: LogReaderMessage::LogLevelInput,
            },
            facility: MyFilter {
                value: String::default(),
                message: LogReaderMessage::FacilityInput,
            },
            log_message: MyFilter {
                value: String::default(),
                message: LogReaderMessage::LogMessageInput,
            },
        }
    }
}

#[derive(Debug)]
pub struct LogReader {
    current_log_file_name: String,

    events: Vec<Record>,
    events_filtered: Vec<usize>,

    // Is dynamically calculated from cursor, so it is possible to get row_offset number
    // that is greater than rows in the table
    highlighted_scroll_offset: usize,
    scroll_value: f64,
    header_height: f32,
    row_height: f32,
    row_padding: f32,
    rows_visible: u32,
    window_height: f32,

    selected_rows: Vec<u64>,
    _mouse_on_table: bool,
    _mouse_y: f32,

    view_detail: Option<ViewDetail>,

    filters: MyFilters,
    //current_screen: ScreenView,
    searching_bar_id: iced::widget::Id,
    searching_bar: bool,
    searching_text: String,

    show_help: bool,
    //loading: u8
}

impl LogReader {
    fn new() -> Self {
        let mut args = env::args();
        args.next();
        let file_name = match args.next() {
            Some(arg) => arg,
            None => panic!("No input found."),
        };

        let events = match Record::read_records() {
            Ok(events) => events,
            Err(e) => panic!("Error while parsing input file. {}", e),
        };
        let events = events;

        Self {
            current_log_file_name: file_name,
            events_filtered: (0..events.len()).collect(),
            events,
            highlighted_scroll_offset: 0,
            scroll_value: 0.0,
            header_height: 75.0,
            row_height: 35.0,
            row_padding: 5.0,
            rows_visible: 0,
            window_height: 0.0,
            selected_rows: vec![],
            _mouse_on_table: false,
            view_detail: None,
            // current_screen: ScreenView::Loading,
            _mouse_y: 0.0,
            filters: MyFilters::default(),
            searching_bar_id: iced::widget::Id::unique(),
            searching_bar: false,
            searching_text: String::default(),
            show_help: false,
        }
    }

    fn title(&self) -> String {
        return format!("Log Reader - {}", self.current_log_file_name);
    }

    fn update(&mut self, msg: LogReaderMessage) -> Task<LogReaderMessage> {
        let ret: Option<Task<LogReaderMessage>>;

        match &msg {
            LogReaderMessage::Event(event) => match event {
                IcedEvent::Keyboard(keyboard_event) => match keyboard_event {
                    iced::keyboard::Event::KeyReleased { key, .. } => match *key {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::F1) => {
                            self.show_help = true;
                            return Task::none();
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)
                            if self.show_help == true =>
                        {
                            self.show_help = false;
                            return Task::none();
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        if self.view_detail.is_some() {
            ret = Some(detail_view::update(self, msg));
        } else {
            ret = Some(table_view::update(self, msg));
        }
        ret.unwrap_or(Task::none())

        /*
         * Showing progress slows the loading of 0.5GB file by 3 seconds...

         if self.loading == 0 {
            self.loading = 1;

            println!("Starting task");
            Task::sip(sipper(async |mut progress| {

                let mut p2 = progress.clone();
                let events = Record::read_records_async("", &mut p2);
                events.await;
                Ok(42)
            }), LogReaderMessage::TaskTest, LogReaderMessage::TaskTest)
        } else {
            ret.unwrap_or(Task::none())
        } */
    }

    fn _build_help_dialog(&self) -> Element<'_, LogReaderMessage> {
        fn txt_red<'a>(txt: &'a str) -> text::Span<'a, std::convert::Infallible> {
            span(txt).color(color!(255, 0, 0))
        }

        fn txt_green<'a>(txt: &'a str) -> text::Span<'a, std::convert::Infallible> {
            span(txt).color(color!(0, 230, 0))
        }

        container(
            container(
                column![
                    text("Help"),
                    row![
                        rich_text([txt_red("/\n"), txt_red("F3\n"), txt_red("Shift + F3\n"),])
                            .on_link_click(never),
                        iced::widget::space().width(Fill),
                        rich_text([
                            txt_green("Open Search Window\n"),
                            txt_green("Go to Next Occurence\n"),
                            txt_green("Go to Previous Occurence\n"),
                        ])
                        .align_x(text::Alignment::Right)
                    ],
                    row![
                        rich_text([txt_red("M\n"), txt_red("n\n"), txt_red("N\n"),]),
                        iced::widget::space().width(Fill),
                        rich_text([
                            txt_green("Mark Row\n"),
                            txt_green("Go to Next Mark\n"),
                            txt_green("Go to Previous Mark\n"),
                        ])
                        .align_x(text::Alignment::Right),
                    ],
                    row![
                        rich_text([
                            txt_red("Page Up\n"),
                            txt_red("Page Down\n"),
                            txt_red("Arrow Up / k / Mouse Scroll Up\n"),
                            txt_red("Arrow Down / j / Mouse Scroll Down\n"),
                        ]),
                        iced::widget::space().width(Fill),
                        rich_text([
                            txt_green("Scroll Page Up\n"),
                            txt_green("Scroll Page Down\n"),
                            txt_green("Highlight Up\n"),
                            txt_green("Highlight Down\n"),
                        ])
                        .align_x(text::Alignment::Right),
                    ]
                ]
                .padding(16)
                .spacing(16),
            )
            .width(500)
            .style(|theme: &Theme| {
                container::Style::default()
                    .background(theme.palette().background)
                    .border(
                        iced::Border::default()
                            .width(2)
                            .color(theme.palette().success),
                    )
            }),
        )
        .center(Fill)
        .into()
    }

    fn view(&self) -> Element<'_, LogReaderMessage> {
        let mut main_view = stack!();
        if self.view_detail.is_some() {
            main_view = main_view.push(detail_view::view(&self));
        } else {
            main_view = main_view.push(table_view::view(&self));
        }

        if self.show_help {
            main_view = main_view.push(self._build_help_dialog());
        }

        main_view.into()
    }

    fn subscribtion(&self) -> Subscription<LogReaderMessage> {
        Subscription::batch([
            event::listen().map(LogReaderMessage::Event),
            window::close_events().map(LogReaderMessage::WindowClosed),
        ])
    }

    fn _scroll(&mut self, change: f64) {
        // lower limit
        if (self.scroll_value - change) <= 0.0 {
            self.scroll_value = 0.0;
            return;
        }

        // upper limit
        if (self.events_filtered.len() - self.rows_visible as usize)
            <= (self.scroll_value - change) as usize
        {
            self.scroll_value =
                (self.events_filtered.len() - self.rows_visible as usize - 1) as f64;
            return;
        }

        self.scroll_value -= change;
    }

    fn _scroll_highlight(&mut self, change: f64) {
        if self.highlighted_scroll_offset as f64 - change <= 0.0 {
            self.highlighted_scroll_offset = 0;
        }

        if (self.events_filtered.len()) <= (self.highlighted_scroll_offset as f64 - change) as usize
        {
            self.highlighted_scroll_offset = self.events_filtered.len() as usize - 1;
            return;
        }

        self.highlighted_scroll_offset = self
            .highlighted_scroll_offset
            .saturating_sub_signed(change as isize);
        if self.highlighted_scroll_offset
            >= (self.scroll_value as usize + self.rows_visible as usize + 1)
        {
            self._scroll(change);
        } else if self.highlighted_scroll_offset < self.scroll_value as usize {
            self._scroll(change);
        }
    }

    fn _get_row_idx_scroll_idx(&self, row_idx: u64) -> Option<u64> {
        if let Some(scroll_idx) = self
            .events_filtered
            .iter()
            .position(|row| self.events.get(*row).unwrap().id == row_idx)
        {
            return Some(scroll_idx as u64);
        }

        None
    }

    fn _get_valid_scrollables(&self) -> Vec<(u64, u64)> {
        let selected_available = self.selected_rows.iter().filter_map(|&row_id| {
            if let Some(scroll_id) = self._get_row_idx_scroll_idx(row_id) {
                Some((scroll_id, row_id))
            } else {
                None
            }
        });

        return selected_available.collect();
    }

    fn _get_next_mark_rev(&self) -> Option<(u64, u64)> {
        let selected_available: Vec<(u64, u64)> = self._get_valid_scrollables();

        for (scroll_id, row_idx) in selected_available.clone().into_iter().rev() {
            if scroll_id >= self.highlighted_scroll_offset as u64 {
                continue;
            }
            return Some((scroll_id, row_idx));
        }

        if let Some(last_mark) = selected_available.last() {
            return Some(*last_mark);
        } else {
            None
        }
    }

    fn _get_next_mark(&self) -> Option<(u64, u64)> {
        let selected_available: Vec<(u64, u64)> = self._get_valid_scrollables();

        for (scroll_id, row_idx) in selected_available.clone() {
            if scroll_id <= self.highlighted_scroll_offset as u64 {
                continue;
            }
            return Some((scroll_id, row_idx));
        }
        if let Some(first_mark) = selected_available.first() {
            return Some(*first_mark);
        } else {
            None
        }
    }

    fn _scroll_to(&mut self, scroll_id_offset: u64) {
        self.scroll_value = scroll_id_offset as f64;
        self.highlighted_scroll_offset = scroll_id_offset as usize;
        self._scroll((self.rows_visible as f64 / 2.0).ceil());
    }

    fn _add_mark(&mut self, row_id: u64) {
        self.selected_rows.push(row_id);
        self.selected_rows.sort();
    }

    fn _remove_mark(&mut self, row_idx: u64) {
        if let Some(idx) = self
            .selected_rows
            .iter()
            .position(|&row_id| row_id == row_idx)
        {
            self.selected_rows.remove(idx);
        }
    }

    fn _switch_mark(&mut self, row_idx: u64) {
        if self.selected_rows.contains(&row_idx) {
            self._remove_mark(row_idx);
        } else {
            self._add_mark(row_idx);
        }
    }

    fn _switch_mark_highlighted_offset(&mut self) {
        let row_idx = self
            .events_filtered
            .get(self.highlighted_scroll_offset)
            .unwrap();

        self._switch_mark(*row_idx as u64);
    }

    fn _scroll_value_to_row_id(&self, scroll_value: u64) -> u64 {
        if let Some(record) = self.events_filtered.get(scroll_value as usize) {
            self.events.get(*record).unwrap().id
        } else {
            0
        }
    }

    fn _find_prev(&mut self) {
        if let Some(&row_idx) = self.events_filtered.get(self.highlighted_scroll_offset) {
            if let Some(row) = self.events_filtered.iter().rev().position(|&item| {
                if row_idx <= item {
                    return false;
                }
                self.events[item].log_message.contains(&self.searching_text)
            }) {
                self._scroll_to((self.events_filtered.len() - row - 1) as u64);
            }
        }
    }

    fn _find_next(&mut self) {
        if let Some(&row_idx) = self.events_filtered.get(self.highlighted_scroll_offset) {
            if let Some(row) = self.events_filtered.iter().position(|&item| {
                if row_idx >= item {
                    return false;
                }
                self.events[item].log_message.contains(&self.searching_text)
            }) {
                self._scroll_to(row as u64);
            }
        }
    }
}
