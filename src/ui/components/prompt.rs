#![allow(unused)]

use iced::Length::Fill;
use iced::keyboard;
use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, row, text};
use smol_str::SmolStr;

use crate::prelude::*;


#[derive(Debug)]
pub enum Message {
    Key(SmolStr),
    None,
}


#[derive(Debug)]
pub enum Action {
    None,
}


#[derive(Debug, Clone)]
pub struct Prompt {
    text: String,
}

type Task = iced::Task<Message>;

// TODO: make my own command prompt component
impl Prompt {

    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let cursor = text!("> {}", self.text);

        container(cursor).into()
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {

        match message {
            Message::None          => Action::None,
            Message::Key(smol_str) => {
                self.text.push_str(smol_str.as_str());

                Action::None
            },
        }
    }

    #[must_use]
    pub fn handle_key_event(&self, key: Key) -> Message {
        trace!("prompt key: {key:?}");

        match key {
            Key::Character(ch) => Message::Key(ch),

            _ => Message::None
        }

    }

}
