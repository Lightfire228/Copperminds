use std::fmt::Display;

use enum_iterator::Sequence;

use crate::{ui::{QueueType, components::sort_queue::SortQueue}, vault::fm::{FmAction, FmStatus}};


type Cm = Command;
type Fa = FmAction;
type Fs = FmStatus;

#[derive(Debug, Clone, Copy, Sequence)]
pub enum Command {
    SetTypeInfo,
    SetAction(FmAction),
    SetStatus(FmStatus),
    DeleteFile,
}

impl Command {

    pub fn all() -> impl Iterator<Item = Self> {
        enum_iterator::all::<Self>()
    }

    pub fn get_label(&self) -> &'static str {
        match &self {
            Cm::SetTypeInfo                  => "type    - info",
            Cm::SetAction(Fa::Todo)          => "action  - todo",
            Cm::SetAction(Fa::Backlog)       => "action  - backlog",
            Cm::SetAction(Fa::Entertainment) => "action  - entertainment",
            Cm::SetAction(Fa::MaybeSomeday)  => "action  - maybe someday",
            Cm::SetAction(Fa::WaitingFor)    => "action  - waiting for",
            Cm::SetStatus(Fs::Completed)     => "status  - complete",
            Cm::SetStatus(Fs::Archived)      => "status  - archived",
            Cm::DeleteFile                   => "command - delete file",
        }
    }

    pub fn get_code(&self, queue: QueueType) -> Option<&'static str> {

        Some(match queue {
            QueueType::Inbox => match &self {
                Cm::SetTypeInfo                  => "i",
                Cm::SetAction(Fa::Todo)          => "t",
                Cm::SetAction(Fa::Backlog)       => "b",
                Cm::SetAction(Fa::Entertainment) => "e",
                Cm::SetAction(Fa::MaybeSomeday)  => "m",
                Cm::SetAction(Fa::WaitingFor)    => "w",
                Cm::SetStatus(Fs::Completed)     => "c",
                Cm::SetStatus(Fs::Archived)      => "a",
                Cm::DeleteFile                   => "d",
            },
            QueueType::Actionables => match &self {
                Cm::SetTypeInfo                  => "i",
                Cm::SetAction(Fa::Todo)          => "t",
                Cm::SetAction(Fa::Backlog)       => "b",
                Cm::SetAction(Fa::Entertainment) => "e",
                Cm::SetAction(Fa::MaybeSomeday)  => "m",
                Cm::SetAction(Fa::WaitingFor)    => "w",
                Cm::SetStatus(Fs::Completed)     => "c",
                Cm::SetStatus(Fs::Archived)      => "a",

                _ => None?,
            },
        })
    }
}


impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::SetTypeInfo  => write!(f, "Set Type Info"),
            Command::SetAction(a) => write!(f, "Set Action {a}"),
            Command::SetStatus(s) => write!(f, "Set Status {s}"),
            Command::DeleteFile   => write!(f, "Delete File"),
        }
    }
}
