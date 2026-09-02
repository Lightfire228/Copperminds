#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::Ok as _;

use file_id::FileId;
use iced::Length::Fill;
use iced::keyboard;
use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, row, text};
use smol_str::SmolStr;

use crate::prelude::*;
use crate::ui::key_event::KeyPressed;


#[derive(Debug)]
pub enum Message {
    None,
    Clear,
}


#[derive(Debug)]
pub enum Action<T> {
    RunCommand(Vec<T>),
    OpenSelected,
}


#[derive(Debug, Clone)]
pub struct Prompt<T: Copy> {
    text:     String,
    commands: HashMap<&'static str, T>,
}

type Task = iced::Task<Message>;


impl<T: Copy> Prompt<T> {

    pub fn new(commands: Vec<MenuCommand<T>>) -> Self {
        Self {
            text:     String::new(),
            commands: commands
                .iter   ()
                .map    (|x| (x.code, x.command))
                .collect(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let cursor = text!("> {}", self.text);

        container(cursor).into()
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Option<Action<T>> {

        match message {
            Message::None => None,

            Message::Clear => {
                self.clear();

                None
            },
        }
    }


    fn clear(&mut self) {
        self.text.clear();
    }

    #[must_use]
    pub fn handle_key_event(&mut self, key: &KeyPressed) -> Option<Action<T>> {
        trace!("prompt key: {key:?}");

        use keyboard::key::Named;

        Some(match &key.key {
            Key::Named(Named::Space) => {

                if self.text.is_empty() {
                    None?
                }

                self.text.push(' ');

                None?
            }
            Key::Named(Named::Backspace) => {

                if key.modifiers.control() {
                    // todo: delete_word_left
                    self.clear();
                }
                else {
                    self.text.pop();
                }

                None?
            }
            Key::Named(Named::Enter) => {

                if self.text.is_empty() {
                    return Some(Action::OpenSelected);
                }

                let commands = self.parse_commands()
                    .inspect_err(|err| warn!("Unable to parse commands: {err}"))
                    .ok()?
                ;

                if commands.is_empty() {
                    None?
                }

                Action::RunCommand(commands)
            }
            Key::Character(ch) => {

                self.text.push_str(ch.as_str());

                None?
            },

            _ => None?
        })
    }

    fn parse_commands(&self) -> Result<Vec<T>, String> {

        self
            .text
            .split    ("")
            .filter   (|x| *x != "")
            .map      (|ch| self.commands
                .get       (ch)
                .copied    ()
                .ok_or_else(|| format!("No command found for '{ch}'"))
            )
            .collect()
    }

}


#[derive(Debug, Clone)]
pub struct MenuCommand<T: Copy> {
    pub code:    &'static str,
    pub name:    &'static str,
    pub command: T,
}
