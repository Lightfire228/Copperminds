#![allow(unused)]

use iced::Length::Fill;
use iced::keyboard;
use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, row, text};
use smol_str::SmolStr;

use crate::prelude::*;


#[derive(Debug)]
pub enum Message {
    None,
}


#[derive(Debug)]
pub enum Action {
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
    pub fn update(&mut self, message: Message) -> Option<Action> {

        match message {
            Message::None => None,
        }
    }

    #[must_use]
    pub fn handle_key_event(&mut self, key: Key) -> Option<Action> {
        trace!("prompt key: {key:?}");

        match key {
            Key::Character(ch) => {
                self.text.push_str(ch.as_str());

                None
            },

            _ => None
        }

    }

}
