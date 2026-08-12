use iced::Length::Fill;
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Button, Column, button, column, pick_list, row, text, tooltip};

use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::command::{IterFilesWith, OpenInObsidian, VaultCommand};
use crate::vault::md_file::{FileId, FileView, MdFile};


#[derive(Debug, Clone)]
pub struct SortQueue {
    pub vault:        Sender<VaultCommand>,
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

    pub fn new(queue_type: QueueType, files: Vec<FileView>, vault: Sender<VaultCommand>) -> Self {

        Self {
            vault,
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

    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {

        match message {
            Message::MoveCursorUp   => {
                if self.index > 0 {
                    self.index -= 1;
                }

                self.open_obsidian()
            },
            Message::MoveCursorDown => {
                self.index += 1;

                self.open_obsidian()
            },
            Message::None         => Action::None,
            Message::NavigateBack => Action::NavigateBack,
        }
    }

    pub fn handle_key_event(&self, key: Key) -> Message {

        type N = keyboard::key::Named;

        match key {
            Key::Named(N::ArrowLeft)   |
            Key::Named(N::Escape)      => Message::NavigateBack,

            Key::Named(N::ArrowUp)     => Message::MoveCursorUp,
            Key::Named(N::ArrowDown)   => Message::MoveCursorDown,

            // TODO:
            Key::Character(_key) => {
                Message::None
            },
            _ => Message::None
        }
    }


    fn open_obsidian(&self) -> Action {

        let Some(file) = self.files.get(self.index) else {
            return Action::None;
        };

        let id = file.id;
        let tx = self.vault.clone();

        let task = Task::future(async move {

            let cmd = OpenInObsidian {
                id,
            };

            send_vault_cmd(tx, cmd).await;

            Message::None
        });

        Action::Run(task)

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
}


#[derive(Debug)]
pub enum Message {
    None,
    MoveCursorUp,
    MoveCursorDown,
    NavigateBack,
}


#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    NavigateBack,
}

impl From<Message> for ui::Message {
    fn from(val: Message) -> Self {
        ui::Message::SortQueue(val)
    }
}
