
use std::collections::HashMap;

use iced::keyboard;
use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{text};

use crate::command_parser::{CommandExpr, CommandGroup, TokenKind, parse, tokenize};
use crate::prelude::*;
use crate::ui::key_event::KeyPressed;


#[derive(Debug)]
pub enum Message {
    Clear,
}


#[derive(Debug)]
pub enum Action {
    RunCommand(Vec<CommandExpr>),
    OpenSelected,
}


#[derive(Debug, Clone)]
pub struct Prompt {
    text:     String,
}

type _Task = iced::Task<Message>;


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
    pub fn handle_key_event(&mut self, key: &KeyPressed) -> Option<Action> {
        trace!("prompt key: {key:?}");

        use keyboard::key::Named;

        Some(match &key.modified_key {
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

    fn parse_commands(&self) -> Result<Vec<CommandExpr>, String> {

        let tokens   = tokenize(&self.text).map_err(|err| format!("Error tokenizing: {err:?}"))?;
        let commands = parse   (tokens)    .map_err(|err| format!("Error parsing: {err:?}"   ))?;

        Ok(commands)

    }

}


#[derive(Debug, Clone, Copy)]
pub struct MenuCommand<T: Copy> {
    pub code:    &'static str,
    pub name:    &'static str,
    pub command: T,
}
