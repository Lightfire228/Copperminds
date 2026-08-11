use iced::Length::Fill;
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Button, Column, button, column, pick_list, row, text, tooltip};

use crate::ui::{Message, QueueType, UIMode};
use crate::vault::command::VaultCommand;
use crate::vault::md_file::{FileId, FileView};


#[derive(Debug, Clone)]
pub struct SortQueue {
    pub queue_type:   QueueType,
    pub files:        Vec<FileView>,
    pub index:        usize,
}

macro_rules! table {
    ($( ($action:ident, $key:literal, $name:literal) ),*$(,)? ) => {[

        $(
            MenuAction {
                key:    $key,
                name:   $name,
                action: VaultAction::$action
            },
        )*
    ]}
}


static NEEDS_TYPE: &'static [MenuAction] = &table!(
    (SetTypeInfo,           "i", "type - info"),
    (SetTypeAction,         "a", "type - action"),
);

static NEEDS_ACTION: &'static [MenuAction] = &table!(
    (SetActionTodo,         "t", "action - todo"),
    (SetActionWaitingFor,   "w", "action - waiting for"),
    // (SetActionCalendar,     "c", "action - calendar"),
    (SetActionProject,      "p", "action - project"),
    (SetActionMaybeSomeday, "m", "action - maybe someday"),
    (SetTypeInfo,           "i", "type   - info"),
    (SetStatusComplete,     "c", "status - complete"),
    (SetStatusArchived,     "a", "status - archived"),
);


impl SortQueue {

    pub fn new(queue_type: QueueType, files: Vec<FileView>) -> Self {

        Self {
            queue_type,
            files,
            index: 0,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {

        let options = match self.queue_type {
            QueueType::NeedsType   => NEEDS_TYPE,
            QueueType::NeedsAction => NEEDS_ACTION,
        }
            .iter()
            .map (|o| text!("{}", o.name).into())
        ;

        container(
            row![
                container(
                    column![
                        text!("{}", self.queue_type),
                        text!("==="),
                        column(options),
                    ]
                )
                    .width  (300)
                    .padding(10)
                ,
                container(
                    column(self
                        .files
                        .iter     ()
                        .enumerate()
                        .map      (|(i, f)| {
                            let x = if i == self.index { "> " } else { "" };

                            text!("{}{}", x, f.name)
                                .wrapping(text::Wrapping::None)
                                .into()
                        })
                    )
                )
                    .width  (Fill)
                    .padding(10)
            ]
            .spacing(40)
        )
        .into()
    }

    // TODO: read this https://jl710.github.io/iced-guide/app_structure/composition.html

    pub fn update(&mut self, message: Message) -> Option<Message> {
        let Message::SortQueueMessage(message) = message else {
            return None;
        };

        match message {
            SortQueueMessage::VaultAction(_vault_action) => todo!(),

            SortQueueMessage::MoveCursorUp   => {
                if self.index > 0 {
                    self.index -= 1;
                }

                self.open_obsidian()
            },
            SortQueueMessage::MoveCursorDown => {
                self.index += 1;

                self.open_obsidian()
            },
        }
    }

    pub fn handle_key_event(&self, key: Key) -> Option<Message> {

        type N = keyboard::key::Named;

        match key {
              Key::Named(N::ArrowLeft)
            | Key::Named(N::Escape)    => Some(Message::NavigateBack),

            Key::Named(N::ArrowUp)     => Some(SortQueueMessage::MoveCursorUp  .into()),
            Key::Named(N::ArrowDown)   => Some(SortQueueMessage::MoveCursorDown.into()),

            // TODO:
            Key::Character(_key) => {
                None
            },
            _ => None
        }
    }


    fn open_obsidian(&self) -> Option<Message> {
        let file = self.files.get(self.index)?;

        Some(Message::OpenInObsidian(file.id))

    }

}

impl From<SortQueue> for UIMode {
    fn from(val: SortQueue) -> Self {
        UIMode::SortQueue(val)
    }
}

#[allow(dead_code)]
struct MenuAction {
    pub key:     &'static str,
    pub name:    &'static str,
    pub action:  VaultAction,
}

#[derive(Debug, Clone)]
pub enum VaultAction {
    SetTypeInfo,
    SetTypeAction,
    SetActionWaitingFor,
    // SetActionCalendar,
    SetActionProject,
    SetActionTodo,
    SetActionMaybeSomeday,
    SetStatusComplete,
    SetStatusArchived,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SortQueueMessage {
    VaultAction(VaultAction),
    MoveCursorUp,
    MoveCursorDown,
}


impl From<SortQueueMessage> for Message {
    fn from(val: SortQueueMessage) -> Self {
        Message::SortQueueMessage(val)
    }
}
