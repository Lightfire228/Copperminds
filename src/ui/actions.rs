use std::fmt::Display;

use enum_iterator::Sequence;

use crate::{ui::{QueueType, components::sort_queue::SortQueue}, vault::fm::{FmAction, FmStatus}};


type Ua = UiAction;
type Fa = FmAction;
type Fs = FmStatus;

#[derive(Debug, Clone, Copy, Sequence)]
pub enum UiAction {
    SetTypeInfo,
    SetAction(FmAction),
    SetStatus(FmStatus),
    FilterBy (FmAction),
    DeleteFile,
}

impl UiAction {

    pub fn all() -> impl Iterator<Item = Self> {
        enum_iterator::all::<Self>()
    }

    pub fn get_label(&self) -> &'static str {
        match &self {
            Ua::SetTypeInfo                  => "type      - info",
            Ua::SetAction(Fa::Todo)          => "action    - todo",
            Ua::SetAction(Fa::Backlog)       => "action    - backlog",
            Ua::SetAction(Fa::Entertainment) => "action    - entertainment",
            Ua::SetAction(Fa::MaybeSomeday)  => "action    - maybe someday",
            Ua::SetAction(Fa::WaitingFor)    => "action    - waiting for",
            Ua::SetStatus(Fs::Completed)     => "status    - complete",
            Ua::SetStatus(Fs::Archived)      => "status    - archived",
            Ua::DeleteFile                   => "command   - delete file",
            Ua::FilterBy (Fa::Todo)          => "filter by - todo",
            Ua::FilterBy (Fa::Backlog)       => "filter by - backlog",
            Ua::FilterBy (Fa::Entertainment) => "filter by - entertainment",
            Ua::FilterBy (Fa::MaybeSomeday)  => "filter by - maybe someday",
            Ua::FilterBy (Fa::WaitingFor)    => "filter by - waiting for",
        }
    }

    pub fn get_code(&self, queue: QueueType) -> Option<&'static str> {

        // 2026-10-09 - Copperminds - TODO - Command input ideas
        Some(match queue {
            QueueType::Inbox => match &self {
                Ua::SetTypeInfo                  => "i",
                Ua::SetAction(Fa::Todo)          => "t",
                Ua::SetAction(Fa::Backlog)       => "b",
                Ua::SetAction(Fa::Entertainment) => "e",
                Ua::SetAction(Fa::MaybeSomeday)  => "m",
                Ua::SetAction(Fa::WaitingFor)    => "w",
                Ua::SetStatus(Fs::Completed)     => "c",
                Ua::SetStatus(Fs::Archived)      => "a",
                Ua::DeleteFile                   => "d",

                _ => None?,
            },
            QueueType::Actionables => match &self {
                Ua::SetTypeInfo                  => "i",
                Ua::SetAction(Fa::Todo)          => "t",
                Ua::SetAction(Fa::Backlog)       => "b",
                Ua::SetAction(Fa::Entertainment) => "e",
                Ua::SetAction(Fa::MaybeSomeday)  => "m",
                Ua::SetAction(Fa::WaitingFor)    => "w",
                Ua::SetStatus(Fs::Completed)     => "c",
                Ua::SetStatus(Fs::Archived)      => "a",

                _ => None?,
            },
        })
    }
}


impl Display for UiAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiAction::SetTypeInfo  => write!(f, "Set Type Info"),
            UiAction::SetAction(a) => write!(f, "Set Action {a}"),
            UiAction::SetStatus(s) => write!(f, "Set Status {s}"),
            UiAction::DeleteFile   => write!(f, "Delete File"),
            UiAction::FilterBy (x) => write!(f, "Filter By {x}"),
        }
    }
}
