use std::usize;

use file_id::FileId;
use iced::Length::Fill;
use iced::keyboard;
use iced::widget::text_input::cursor;
use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, row, text};

use crate::collections::Files;
use crate::prelude::*;
use crate::vault::md_file::FileView;


#[derive(Debug, Clone)]
pub struct FileList {
    files:  Vec<FileView>,
    cursor: usize,
}


#[derive(Debug)]
pub enum Message {
    None,
    LoadFiles(Files),
    MoveCursorUp,
    MoveCursorDown,

}


#[derive(Debug)]
pub enum Action {
    None,
    Selected(FileId)
}

type Task = iced::Task<Message>;

impl FileList {

    pub fn new() -> Self {
        Self {
            files:  vec![],
            cursor: 0,
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
                    text!("{}", f.name).into(),
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
    pub fn update(&mut self, message: Message) -> Action {

        match message {
            Message::None         => Action::None,
            Message::LoadFiles(files) => {
                self.files = files.into();

                Action::None
            }
            Message::MoveCursorUp   => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }

                self.on_selected()
            },
            Message::MoveCursorDown => {
                if self.cursor < self.files.len() -1 {
                    self.cursor += 1;
                }

                self.on_selected()

            },
        }
    }

    fn on_selected(&self) -> Action {
        self
            .get_selected()
            .map      (|f| Action::Selected(f.id))
            .unwrap_or(    Action::None)

    }

    #[must_use]
    pub fn handle_key_event(&self, key: &Key) -> Message {

        type N = keyboard::key::Named;

        match key {
            Key::Named(N::ArrowUp)     => Message::MoveCursorUp,
            Key::Named(N::ArrowDown)   => Message::MoveCursorDown,

            _ => Message::None
        }
    }

    pub fn get_selected(&self) -> Option<&FileView> {
        self
            .files
            .get(self.cursor)
    }

}
