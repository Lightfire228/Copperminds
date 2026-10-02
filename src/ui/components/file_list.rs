use std::marker::PhantomData;
use std::usize;

use file_id::FileId;
use iced::{Renderer, color};
use iced::advanced::{Widget};
use iced::{Alignment, Color, Length, Size, border, keyboard};
use iced::{Element, keyboard::Key};
use iced::widget::{container, sensor, text};
use log::trace;

use crate::collections::Files;
use crate::ui::key_event::KeyPressed;
use crate::ui::table::RowInfo;
use crate::ui::{table};
use crate::vault::FileView;


#[derive(Debug, Clone)]
pub struct FileList {
    files:    Vec<FileView>,
    cursor:   usize,
}


#[derive(Debug)]
pub enum Message {
    LoadFiles(Files),
}


#[derive(Debug)]
pub enum Action {
    Selected(FileId)
}

type _Task = iced::Task<Message>;

impl FileList {

    pub fn new() -> Self {
        Self {
            files:    vec![],
            cursor:   0,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {

        // type T<'a> = (usize, &'a FileView);

        macro_rules! get_cursor {
            ($row:expr, $cursor:expr) => {{
                let cursor = if $cursor == $row {
                    "> "
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
            // .skip     (self.cursor.scroll)
            // .take     (self.cursor.visible.max(1))
            // .enumerate()
        ;

        let cursor = self.cursor;

        type Fv = FileView;
        type Ri = RowInfo;
        let table: Element<'_, Message> = table::Table::new(
            &[
                Box::new(move |_:    &Fv, info: Ri| get_cursor!  (info.row, cursor)),
                Box::new(move |file: &Fv, _:    Ri| get_file_name(file)),
            ],
            files,
        )
            .into()
        ;

        table.explain(color!(125, 255, 255))

    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Option<Action> {

        match message {
            Message::LoadFiles(files) => {
                self.files = files.into();

                self.files.sort_by_key(|x| x.id);

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

    fn on_navigate(&mut self, direction: Direction, count: usize) -> Option<Action> {
        match direction {
            Direction::Up   => self.cursor = self.cursor.saturating_sub(count),
            Direction::Down => self.cursor = (self.cursor + count).min(self.files.len() -1),
        }

        self.on_selected()
    }

    pub fn get_selected(&self) -> Option<&FileView> {
        self
            .files
            .get(self.cursor)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

}


enum Direction {
    Up,
    Down,
}
