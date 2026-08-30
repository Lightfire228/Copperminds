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
pub enum Action {
    RunCommand(Vec<Command>),
    OpenSelected,
}


#[derive(Debug, Clone)]
pub struct Prompt {
    text: String,
}

type Task = iced::Task<Message>;


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

                let commands = self.validate_commands(commands)
                    .inspect_err(|err| warn!("{err}"))
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

    fn parse_commands(&self) -> Result<Vec<Command>, String> {

        let command_map: HashMap<&'static str, Command> = COMMANDS
            .iter   ()
            .map    (|x| (x.code, x.command))
            .collect()
        ;

        self
            .text
            .split    ("")
            .filter   (|x| *x != "")
            .map      (|ch| command_map
                .get       (ch)
                .copied    ()
                .ok_or_else(|| format!("No command found for '{ch}'"))
            )
            .collect()
    }

    fn validate_commands(&self, commands: Vec<Command>) -> Result<Vec<Command>, String> {

        let mut delete     = vec![];
        let mut not_delete = vec![];

        for cmd in commands.iter() {
            match cmd {
                Command::DeleteFile => delete    .push(cmd),
                _                   => not_delete.push(cmd),
            }
        }

        if !delete.is_empty() && !not_delete.is_empty() {
            Err(
                format!("Incompatible commands, Delete and {:?}", not_delete)
            )?
        };

        Ok(commands)
    }

}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    SetTypeInfo,
    SetActionTodo,
    SetActionWaitingFor,
    SetActionProject,
    SetActionMaybeSomeday,
    SetStatusComplete,
    SetStatusArchived,
    DeleteFile,
}


#[derive(Debug)]
pub struct MenuCommand {
    pub code:    &'static str,
    pub name:    &'static str,
    pub command: Command,
}

macro_rules! table {
    ($( ($command:ident, $code:literal, $name:literal) ),*$(,)? ) => {[

        $(
            MenuCommand {
                code:    $code,
                name:    $name,
                command: Command::$command
            },
        )*
    ]}
}


pub static COMMANDS: &'static [MenuCommand] = &table!(
    (SetTypeInfo,           "i", "type    - info"),
    (SetActionTodo,         "t", "action  - todo"),
    (SetActionWaitingFor,   "w", "action  - waiting for"),
    (SetActionProject,      "p", "action  - project"),
    (SetActionMaybeSomeday, "m", "action  - maybe someday"),
    (SetStatusComplete,     "c", "status  - complete"),
    (SetStatusArchived,     "a", "status  - archived"),
    (DeleteFile,            "d", "command - delete file"),
);


impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::SetTypeInfo           => write!(f, "Set Type Info"),
            Command::SetActionTodo         => write!(f, "Set Action Todo"),
            Command::SetActionWaitingFor   => write!(f, "Set Action Waiting For"),
            Command::SetActionProject      => write!(f, "Set Action Project"),
            Command::SetActionMaybeSomeday => write!(f, "Set Action Maybe Someday"),
            Command::SetStatusComplete     => write!(f, "Set Status Complete"),
            Command::SetStatusArchived     => write!(f, "Set Status Archived"),
            Command::DeleteFile            => write!(f, "Delete File"),
        }
    }
}
