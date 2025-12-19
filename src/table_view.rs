use std::{ops::Add, str::FromStr};

use chrono::{DateTime, Local};
use iced::{
    Element, Event as IcedEvent, Font,
    Length::Fill,
    Task, Theme, Vector, color, font, keyboard, mouse,
    widget::{
        Column, Container, Row, column, container, mouse_area, operation::focus, row, slider,
        stack, text, text_editor, text_input, vertical_slider,
    },
    window,
};

use crate::record::Record;
use crate::{LogColumn, MyFilter, log_message::LogReaderMessage};

use crate::LogReader;
use crate::ViewDetail;

const ROW_HIGHLIGHT_BG_COLOR_L: iced::Background =
    iced::Background::Color(iced::color!(188, 249, 84));
const ROW_HIGHLIGHT_BG_COLOR_D: iced::Background =
    iced::Background::Color(iced::color!(105, 163, 6));

const ROW_HIGHLIGHT_TXT_COLOR_L: iced::Color = iced::Color::BLACK;
const ROW_HIGHLIGHT_TXT_COLOR_D: iced::Color = iced::Color::BLACK;

const ROW_WARNING_BG_COLOR_L: iced::Background = iced::Background::Color(iced::color!(163, 140, 6));
const ROW_WARNING_BG_COLOR_D: iced::Background =
    iced::Background::Color(iced::color!(249, 225, 84));

const ROW_WARNING_TXT_COLOR_L: iced::Color = iced::Color::WHITE;
const ROW_WARNING_TXT_COLOR_D: iced::Color = iced::Color::BLACK;

fn check_str_filter(val: &MyFilter) -> Option<&str> {
    if !val.value.is_empty() {
        return Some(&val.value);
    }
    None
}

fn check_num_filter<N>(val: &MyFilter) -> Option<N>
where
    N: Add<Output = N> + FromStr,
{
    if !val.value.is_empty() {
        if let Ok(val) = val.value.parse::<N>() {
            return Some(val);
        }
    }
    None
}

pub fn apply_filter(table: &mut LogReader) {
    let mut filters: Vec<Box<dyn Fn(&Record) -> bool>> = vec![];

    let t_filters = &table.filters;

    if !t_filters.date.value.is_empty() {
        if let Ok(date) = table.filters.date.value.parse::<DateTime<Local>>() {
            filters.push(Box::new(move |item: &Record| item.date.ge(&date)));
        }
    }

    if let Some(txt) = check_str_filter(&t_filters.computer_name) {
        filters.push(Box::new(move |item: &Record| {
            item.computer_name.contains(&txt)
        }));
    }

    if let Some(num) = check_num_filter(&t_filters.process_id) {
        filters.push(Box::new(move |item: &Record| item.process_id == num));
    }

    if let Some(txt) = check_str_filter(&t_filters.process_user) {
        filters.push(Box::new(move |item: &Record| {
            item.process_user.contains(&txt)
        }));
    }

    if let Some(txt) = check_str_filter(&t_filters.module_name) {
        filters.push(Box::new(move |item: &Record| {
            item.module_name.contains(&txt)
        }));
    }

    if let Some(num) = check_num_filter(&t_filters.message_id) {
        filters.push(Box::new(move |item: &Record| item.message_id == num));
    }

    if let Some(num) = check_num_filter(&t_filters.log_level) {
        filters.push(Box::new(move |item: &Record| item.log_level == num));
    }

    if let Some(num) = check_num_filter(&t_filters.facility) {
        filters.push(Box::new(move |item: &Record| item.facility == num));
    }

    if let Some(txt) = check_str_filter(&t_filters.log_message) {
        filters.push(Box::new(move |item: &Record| {
            item.log_message.contains(&txt)
        }));
    }

    table.events_filtered = table
        .events
        .iter()
        .filter(|&item| {
            for f in filters.iter() {
                if !f(item) {
                    return false;
                }
            }
            return true;
        })
        .map(|r| r.id as usize)
        .collect();
}

pub fn update(table: &mut LogReader, msg: LogReaderMessage) -> Task<LogReaderMessage> {
    match msg {
        /* LogReaderMessage::Change(x) => {
            let events_len = table.events.len();
            if let Some(row) = table.events.get_mut(events_len - 1) {
                // row.name = "Something".to_owned();
            }
        } */
        LogReaderMessage::ScrollChanged(value) => {
            let target_change = table.events_filtered.len() as f64 - (value - 1.0);
            table._scroll_highlight(table.scroll_value - target_change);
        }

        LogReaderMessage::TableEntered => {
            table._mouse_on_table = true;
        }
        LogReaderMessage::TableLeft => {
            table._mouse_on_table = false;
        }
        LogReaderMessage::Event(event) => match event {
            IcedEvent::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => match key {
                    iced::keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        table._scroll_highlight(-1.0)
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        table._scroll_highlight(1.0)
                    }
                    iced::keyboard::Key::Character(c) if c == "j" => {
                        table._scroll_highlight(-1.0);
                    }
                    iced::keyboard::Key::Character(c) if c == "k" => {
                        table._scroll_highlight(1.0);
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                        table._scroll_highlight(-(table.rows_visible as f64))
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                        table._scroll_highlight(table.rows_visible as f64)
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::Enter)
                        if table.searching_bar == false =>
                    {
                        let selected_row = table
                            .events
                            .get(
                                *table
                                    .events_filtered
                                    .get(table.highlighted_scroll_offset)
                                    .unwrap(),
                            )
                            .unwrap();

                        table.view_detail = Some(ViewDetail {
                            record: selected_row.clone(),
                            content: text_editor::Content::with_text(&selected_row.log_message),
                        });
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::F3) if modifiers.shift() => {
                        table._find_prev();
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::F3) => {
                        table._find_next();
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::Escape)
                        if table.searching_bar == true =>
                    {
                        table.searching_bar = false;
                    }
                    iced::keyboard::Key::Named(keyboard::key::Named::Enter)
                        if table.searching_bar == true =>
                    {
                        table.searching_bar = false;
                        table._find_next();
                    }
                    iced::keyboard::Key::Character(c)
                        if c == "/" && table.searching_bar == false =>
                    {
                        table.searching_bar = true;
                        return focus(table.searching_bar_id.clone());
                    }
                    iced::keyboard::Key::Character(c) if c == "n" || c == "N" => {
                        if table.selected_rows.len() == 0 {
                            return Task::none();
                        }
                        let mark_pos;
                        if modifiers.shift() {
                            mark_pos = table._get_next_mark_rev();
                        } else {
                            mark_pos = table._get_next_mark();
                        }
                        if let Some((scroll_idx, _row_idx)) = mark_pos {
                            table._scroll_to(scroll_idx);
                        }
                    }
                    iced::keyboard::Key::Character(c) if c == "m" => {
                        table._switch_mark_highlighted_offset();
                    }
                    _ => (),
                },
                keyboard::Event::KeyReleased { key, .. } => match key {
                    iced::keyboard::Key::Named(keyboard::key::Named::Escape)
                        if table.searching_bar =>
                    {
                        table.searching_bar = false;
                    }
                    _ => (),
                },
                _ => (),
            },
            IcedEvent::Mouse(mouse_event) => match mouse_event {
                mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { y, .. },
                } => {
                    table._scroll_highlight(y as f64);
                    table._scroll(y as f64);
                    // scroll here
                }
                mouse::Event::CursorMoved { position } => {
                    table._mouse_y = position.y - table.header_height;
                    if table._mouse_y < 0.0 {
                        return Task::none();
                    }

                    // table.highlighted = (pos_y / (table.rows_visible as f32 - (30.0 / table.row_height).ceil())) as u64;
                    /* table.highlighted_row_id = table._scroll_value_to_row_id(((table._mouse_y / table.row_height as f32) as f64
                    + (table.scroll_value)) as u64); */
                    table.highlighted_scroll_offset =
                        ((table._mouse_y / table.row_height as f32) as f64 + (table.scroll_value))
                            as usize;
                }
                /* mouse::Event::ButtonReleased(btn) => {
                    if table._mouse_y > 0.0 && table._mouse_on_table && btn == mouse::Button::Left {
                        table._switch_mark(
                            *table
                                .events_filtered
                                .get(table.highlighted_row_offset)
                                .unwrap() as u64,
                        );
                    }
                } */
                _ => (),
            },
            IcedEvent::Window(window_event) => match window_event {
                window::Event::Resized(iced::Size { height, .. }) => {
                    table.window_height = height;
                    table.rows_visible =
                        ((height - (table.header_height + 30.0)) / table.row_height) as u32
                }
                _ => (),
            },
            _ => (),
        },

        LogReaderMessage::IDInput(txt) => {
            table.filters.log_name.value = txt;
            apply_filter(table);
            // apply_filter(table, |&r| r.log_name.contains(&txt));
        }
        LogReaderMessage::DateInput(txt) => {
            table.filters.date.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::ComputerNameInput(txt) => {
            table.filters.computer_name.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::ProcessIDInput(txt) => {
            table.filters.process_id.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::ProcessUserInput(txt) => {
            table.filters.process_user.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::ModuleNameInput(txt) => {
            table.filters.module_name.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::MessageIDInput(txt) => {
            table.filters.message_id.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::LogLevelInput(txt) => {
            table.filters.log_level.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::FacilityInput(txt) => {
            table.filters.facility.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::LogMessageInput(txt) => {
            table.filters.log_message.value = txt;
            apply_filter(table);
        }
        LogReaderMessage::SearchInputChanged(txt) => {
            table.searching_text = txt;
        }
        LogReaderMessage::WindowClosed(_window_id) => {}
        _ => (),
    }

    Task::none()
}

pub fn view(table: &LogReader) -> Element<'_, LogReaderMessage> {
    let container_table_slider = _build_table(&table);

    let mut comp_stack = stack!(container_table_slider.width(Fill).height(Fill),);

    if table.searching_bar {
        comp_stack = comp_stack.push(_build_search_window(&table));
    }

    comp_stack.into()
}

fn _build_search_window(log_table: &LogReader) -> Container<'_, LogReaderMessage> {
    container(
        container(
            column![
                text("Find in Log Message"),
                text_input("", &log_table.searching_text)
                    .on_input(LogReaderMessage::SearchInputChanged)
                    .id(log_table.searching_bar_id.clone())
                    .width(500),
            ]
            .padding(16)
            .spacing(16),
        )
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
}

fn _build_table(log_table: &LogReader) -> Row<'_, LogReaderMessage> {
    let header_impl = move |header, filter_ref: &MyFilter| {
        let filter_input = text_input("", &filter_ref.value).on_input(filter_ref.message);

        container(column![
            text(header)
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
                .wrapping(text::Wrapping::None)
                .height(Fill),
            filter_input,
        ])
        .clip(true)
        .height(log_table.header_height)
    };

    let body_modifier = |record: &Record, input_some| {
        let Record { id, facility, .. } = *record;

        container(input_some)
            .clip(true)
            .width(Fill)
            .height(log_table.row_height)
            .padding(log_table.row_padding)
            .style(move |theme: &Theme| {
                let mut bg_color: Option<iced::Background> = None;
                let mut txt_color: Option<iced::Color> = None;

                let is_dark = theme.extended_palette().is_dark;

                if facility == 3 {
                    //warning
                    if is_dark {
                        bg_color = Some(ROW_WARNING_BG_COLOR_D);
                        txt_color = Some(ROW_WARNING_TXT_COLOR_D);
                    } else {
                        bg_color = Some(ROW_WARNING_BG_COLOR_L);
                        txt_color = Some(ROW_WARNING_TXT_COLOR_L);
                    }
                } else if facility == 4 {
                    //error
                    bg_color = Some(iced::Background::Color(iced::color!(240, 44, 44)));
                    txt_color = Some(iced::Color::BLACK);
                }

                if log_table.selected_rows.contains(&id) {
                    bg_color = Some(iced::Background::Color(iced::color!(0, 255, 0)));
                    txt_color = Some(iced::color!(0, 0, 0));
                    // txt_color = Some(theme.palette().background);
                }

                if let Some(sel_id) = log_table
                    .events_filtered
                    .get(log_table.highlighted_scroll_offset)
                {
                    if id as usize == *sel_id {
                        if is_dark {
                            bg_color = Some(ROW_HIGHLIGHT_BG_COLOR_D);
                            txt_color = Some(ROW_HIGHLIGHT_TXT_COLOR_D)
                        } else {
                            bg_color = Some(ROW_HIGHLIGHT_BG_COLOR_L);
                            txt_color = Some(ROW_HIGHLIGHT_TXT_COLOR_L);
                        }
                    }
                }

                container::Style {
                    background: bg_color,
                    text_color: txt_color,
                    shadow: iced::Shadow {
                        color: color!(0),
                        offset: Vector { x: 0.0, y: 0.0 },
                        blur_radius: 0.5,
                    },
                    ..container::Style::default()
                }
            })
    };

    let items = log_table.events_filtered.iter();

    let scroller_value = items.len() as f64 - log_table.scroll_value;
    let slider = vertical_slider(
        // (log_table.rows_visible as f64)..=((items.len() + 1) as f64),
        (log_table.rows_visible as f64)..=((items.len() + 1) as f64),
        scroller_value,
        LogReaderMessage::ScrollChanged,
    )
    .style(|theme, status| vertical_slider::Style {
        rail: slider::Rail {
            backgrounds: (
                iced::Background::Color(iced::color!(128, 128, 128)),
                iced::Background::Color(iced::color!(128, 128, 128)),
            ),
            width: 2.0,
            border: iced::Border::default(),
        },
        ..vertical_slider::default(theme, status)
    });

    let columns_def = [
        LogColumn::new(
            header_impl("ID", &log_table.filters.log_name),
            |r: &Record| body_modifier(r, text(&r.id)),
            100,
        ),
        LogColumn::new(
            header_impl("Date", &log_table.filters.date),
            |r: &Record| {
                body_modifier(
                    r,
                    text(r.date.format("%d. %m. %Y %H:%M:%S%.3f").to_string())
                        .wrapping(text::Wrapping::None),
                )
            },
            200,
        ),
        LogColumn::new(
            header_impl("Computer\nName", &log_table.filters.computer_name),
            |r: &Record| body_modifier(r, text(&r.computer_name)),
            120,
        ),
        LogColumn::new(
            header_impl("Process ID", &log_table.filters.process_id),
            |r: &Record| body_modifier(r, text(&r.process_id)),
            100,
        ),
        LogColumn::new(
            header_impl("Process\nUser", &log_table.filters.process_user),
            |r: &Record| body_modifier(r, text(&r.process_user)),
            100,
        ),
        LogColumn::new(
            header_impl("Module\nName", &log_table.filters.module_name),
            |r: &Record| body_modifier(r, text(&r.module_name)),
            120,
        ),
        LogColumn::new(
            header_impl("Message ID", &log_table.filters.message_id),
            |r: &Record| body_modifier(r, text(&r.message_id)),
            100,
        ),
        LogColumn::new(
            header_impl("Log\nLevel", &log_table.filters.log_level),
            |r: &Record| body_modifier(r, text(&r.log_level)),
            80,
        ),
        LogColumn::new(
            header_impl("Facility", &log_table.filters.facility),
            |r: &Record| body_modifier(r, text(&r.facility)),
            70,
        ),
        LogColumn::new(
            header_impl("Log\nMessage", &log_table.filters.log_message),
            |r: &Record| body_modifier(r, text(&r.log_message).wrapping(text::Wrapping::None)),
            Fill,
        ),
    ];

    let (mut columns, views): (Vec<_>, Vec<_>) = columns_def
        .into_iter()
        .map(|item| ((vec![item.header], item.width), item.view))
        .collect();

    for (i, item_row) in items.enumerate() {
        if (i as f64) >= (log_table.scroll_value)
            && (i as f64) <= ((log_table.scroll_value) + log_table.rows_visible as f64)
        {
            let rec = log_table.events.get(*item_row).unwrap();

            for (idx, (col, _width)) in columns.iter_mut().enumerate() {
                col.push((views[idx])(rec));
            }
        }
    }

    let view_row = Row::with_children(
        columns
            .into_iter()
            .map(|(column_containers, width)| {
                let column = Column::with_children(
                    column_containers
                        .into_iter()
                        .map(Into::into)
                        .collect::<Vec<_>>(),
                )
                .width(width);
                column.into()
            })
            .collect::<Vec<_>>(),
    );

    row![
        container(
            mouse_area(view_row)
                .on_enter(LogReaderMessage::TableEntered)
                .on_exit(LogReaderMessage::TableLeft)
        )
        .clip(true),
        container(slider).align_right(10)
    ]
}
