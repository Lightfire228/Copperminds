use iced::Length::Fill;
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Button, Column, button, column, pick_list, row, text, tooltip};

use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::command::{IterFilesWith, OpenInObsidian, SetProperty, VaultCommand};
use crate::vault::fm::{FmAction, FmProperty, FmStatus, FmType, GetKey};
use crate::vault::md_file::{FileView, MdFile};


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

    pub fn new(queue_type: QueueType, vault: Sender<VaultCommand>) -> (Self, Task<Message>) {
        (
            Self {
                queue_type,
                vault: vault.clone(),
                files: Vec::new(),
                index: 0,
            },
            Task::future(async move {
                let files = load_files(vault, queue_type).await;

                Message::LoadFiles(files)
            })

        )
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

            Message::LoadFiles(files) => {
                self.files = files;

                Action::None
            },
            Message::VaultAction(action) => Action::Run(self.handle_vault_action(action))
        }
    }

    pub fn handle_key_event(&self, key: Key) -> Message {

        type N = keyboard::key::Named;


        match key {
            Key::Named(N::ArrowLeft)   |
            Key::Named(N::Escape)      => Message::NavigateBack,

            Key::Named(N::ArrowUp)     => Message::MoveCursorUp,
            Key::Named(N::ArrowDown)   => Message::MoveCursorDown,

            Key::Character(key) => {
                let list = match self.queue_type {
                    QueueType::NeedsType   => NEEDS_TYPE,
                    QueueType::NeedsAction => NEEDS_ACTION,
                };

                let action = list.iter().filter(|a| a.key == key.as_str()).next();

                let Some(action) = action else {
                    return Message::None;
                };

                Message::VaultAction(action.action)
            },
            _ => Message::None
        }
    }

    fn handle_vault_action(&mut self, action: VaultAction) -> Task<Message> {

        let tx = self.vault.clone();
        let (prop, value) = action.get_value();

        let id = self.files[self.index].id;


        Task::future(async move {

            // TODO: this doesn't actually write the changes to disk
            // i wanna get the file watch in place before then
            send_vault_cmd(tx, SetProperty {
                id,
                prop,
                value,
            })
                .await
            ;

            Message::None
        })

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
#[derive(Debug)]
struct MenuAction {
    pub key:     &'static str,
    pub name:    &'static str,
    pub action:  VaultAction,
}

#[derive(Debug, Clone, Copy)]
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


#[derive(Debug)]
pub enum Message {
    None,
    LoadFiles(Vec<FileView>),
    MoveCursorUp,
    MoveCursorDown,
    NavigateBack,
    VaultAction(VaultAction),
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


async fn load_files(vault: Sender<VaultCommand>, queue: QueueType) -> Vec<FileView> {

    let cmd = match queue {
        QueueType::NeedsType   => |f: &MdFile| f.needs_type(),
        QueueType::NeedsAction => |f: &MdFile| f.needs_action_type(),
    };

    send_vault_cmd(
        vault,
        IterFilesWith {
            filter: cmd,
        }
    )
    .await
}


impl VaultAction {
    fn get_value(&self) -> (FmProperty, String) {
        match self {
            VaultAction::SetTypeInfo           => (FmProperty::Type,   FmType  ::Info        .get_key()),
            VaultAction::SetTypeAction         => (FmProperty::Type,   FmType  ::Action      .get_key()),
            VaultAction::SetActionWaitingFor   => (FmProperty::Action, FmAction::WaitingFor  .get_key()),
            VaultAction::SetActionProject      => (FmProperty::Action, FmAction::Project     .get_key()),
            VaultAction::SetActionTodo         => (FmProperty::Action, FmAction::Todo        .get_key()),
            VaultAction::SetActionMaybeSomeday => (FmProperty::Action, FmAction::MaybeSomeday.get_key()),
            VaultAction::SetStatusComplete     => (FmProperty::Status, FmStatus::Completed   .get_key()),
            VaultAction::SetStatusArchived     => (FmProperty::Status, FmStatus::Archived    .get_key()),
        }
    }
}
