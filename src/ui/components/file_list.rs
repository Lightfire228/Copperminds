use std::marker::PhantomData;
use std::usize;

use file_id::FileId;
use iced::advanced::overlay::Element as AdvancedElement;
use iced::{Renderer, color};
use iced::advanced::{Widget};
use iced::{Alignment, Color, Length, Size, border, keyboard};
use iced::{Element, keyboard::Key};
use iced::widget::{container, sensor, text};
use log::{debug, trace};

use crate::collections::Files;
use crate::ui::key_event::KeyPressed;
use crate::ui::table::{ColInfo, Row, Table};
use crate::ui::{table};
use crate::vault::FileView;


pub struct FileList {
    files:        Vec<FileView>,
    cursor:       usize,
    rows_visible: usize,
    scroll:       usize,
}


#[derive(Debug)]
pub enum Message {
    LoadFiles  (Files),
    RowsVisible(usize),
}


#[derive(Debug)]
pub enum Action {
    Selected(FileId)
}

type _Task = iced::Task<Message>;

impl FileList {



    pub fn new() -> Self {
        Self {
            files:        vec![],

            scroll:       0,
            cursor:       0,
            rows_visible: 0,
        }
    }

    pub fn index(&self) -> usize {
        self.scroll + self.cursor
    }

    pub fn view(&self) -> Element<'_, Message> {

        macro_rules! get_cursor {
            ($row:expr, $carat:expr) => {{
                let cursor = if self.cursor == $row {
                    $carat
                }
                else {
                    ""
                };

                text!("{cursor}")
                    .wrapping(text::Wrapping::None)
                    .into()
            }};
        }

        fn get_file_name<'a>(file: &FileView) -> Element<'a, Message> {
            text!("{}", file.name)
                .wrapping(text::Wrapping::None)
                .into()
        }

        let files = self
            .files
            .iter     ()
            .skip     (self.scroll)
            .take     (self.rows_visible)
        ;

        let table: Element<'_, Message> = table::Table::new(
            25.0,
            &[
                ColInfo { size: Length::Fixed(20.0), col: &|row: Row<FileView>| get_cursor!  (row.y, "> ")},
                ColInfo { size: Length::Fill,        col: &|row: Row<FileView>| get_file_name(row.data)},
                ColInfo { size: Length::Fixed(20.0), col: &|row: Row<FileView>| get_cursor!  (row.y, " <")},
            ],
            files,
            Message::RowsVisible,
        )
            .into()
        ;

        // table.explain(color!(125, 255, 255))
        table

    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Option<Action> {

        match message {
            Message::LoadFiles(files) => {
                self.files = files.into();

                self.files.sort_by_key(|x| x.id);

                self.scroll = self.scroll.min(self.calc_max_scroll());
                self.cursor = self.cursor.min(self.calc_max_cursor());

                None
            },

            Message::RowsVisible(rows) => {
                self.rows_visible = rows;

                None
            }
        }
    }

    fn on_selected(&mut self) -> Option<Action> {
        self
            .get_selected()
            .map(|f| Action::Selected(f.id))
    }

    #[must_use]
    pub fn handle_key_event(&mut self, key: &KeyPressed) -> Option<Action> {

        type N = keyboard::key::Named;

        match key.key {
            Key::Named(N::ArrowUp)     => self.on_navigate(Direction::Up,    1),
            Key::Named(N::ArrowDown)   => self.on_navigate(Direction::Down,  1),
            Key::Named(N::PageUp)      => self.on_navigate(Direction::Up,   20),
            Key::Named(N::PageDown)    => self.on_navigate(Direction::Down, 20),

            _ => None
        }
    }

    #[must_use]
    fn on_navigate(&mut self, direction: Direction, count: usize) -> Option<Action> {
        let prv = self.cursor;

        let max_index  = self.calc_max_index();
        let max_cursor = self.calc_max_cursor();

        match direction {
            Direction::Up   => {
                self.cursor = self.cursor.saturating_sub(count);

                if prv < count {
                    self.scroll = self.scroll.saturating_sub(count - prv);
                }
            },
            Direction::Down => {
                let count = if prv + count > max_cursor {
                    let diff = prv + count - max_cursor;

                    self.scroll = (self.scroll + diff).min(max_index - max_cursor);

                    count - diff
                }
                else {
                   count
                };

                self.cursor = (self.cursor + count).min(max_index);

            }
        }

        self.on_selected()
    }

    fn calc_max_index(&self) -> usize {
        self.files.len().saturating_sub(1)
    }

    fn calc_max_cursor(&self) -> usize {
        self
            .calc_max_index()
            .min(self.rows_visible.saturating_sub(1))
    }

    fn calc_max_scroll(&self) -> usize {
        self
            .calc_max_index()
            .saturating_sub(self.calc_max_cursor())
    }

    pub fn get_selected(&self) -> Option<&FileView> {
        self
            .files
            .get(self.index())
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }
}


enum Direction {
    Up,
    Down,
}
