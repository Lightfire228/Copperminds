use std::usize;

use file_id::FileId;
use iced::Length::Fill;
use iced::keyboard;
use iced::widget::text_editor::KeyPress;
use iced::widget::text_input::cursor;
use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, row, text};

use crate::collections::Files;
use crate::prelude::*;
use crate::ui::key_event::KeyPressed;
use crate::vault::md_file::FileView;


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

        let (cursors, files): (Vec<_>, Vec<_>) = self
            .files
            .iter     ()
            .enumerate()
            .map      (|(i, f)| {
                let x = if i == self.cursor { "> " } else { "" };

                (
                    text!("{}", x)     .into(),
                    text!("{}", f.name)
                        .wrapping(text::Wrapping::None)
                        .into()
                    ,
                )
            })
            .unzip()
        ;

        row![
            container(column(cursors))
                .padding([10, 0])
            ,
            container(
                column(files)
            )
                .width  (Fill)
                .padding([10, 0])
        ]
            .into()

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
