use file_id::FileId;
use iced::Length::Fill;
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{column, row, text};

use crate::ui::components::file_list::{self, FileList};
use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::command::{IterFilesWith, OpenInObsidian, SetProperty, VaultCommand, VaultUpdate};
use crate::vault::fm::{FmAction, FmProperty, FmStatus, FmType, GetKey};
use crate::vault::md_file::{FileView, MdFile};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct SortQueue {
    vault:        Sender<VaultCommand>,
    queue_type:   QueueType,
    index:        usize,

    file_list:    FileList,
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

        let (list_state, list_task) = FileList::new();

        (
            Self {
                queue_type,
                vault:     vault.clone(),
                index:     0,
                file_list: list_state,
            },
            Task::batch([
                Task::perform(load_files(vault, queue_type), Message::LoadFiles),
                list_task.map(|_| Message::None),
            ])

        )
    }

    pub fn view(&self) -> Element<'_, Message> {

        let options = match self.queue_type {
            QueueType::NeedsType   => NEEDS_TYPE,
            QueueType::NeedsAction => NEEDS_ACTION,
        }
            .iter()
            .map (|o| text!("{} - {}", o.key, o.name).into())
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
                self.file_list.view().map(Message::FileList),
            ]
            .spacing(40)
        )
        .into()
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {

        match message {
            Message::FileList(message) => {
                let action = self.file_list.update(message);
                self.handle_file_list_action(action)
            }            ,
            Message::LoadFiles(files) => {
                let message = file_list::Message::Files(files);

                let action = self.file_list.update(message);
                self.handle_file_list_action(action)
            }
            Message::None         => Action::None,
            Message::NavigateBack => Action::NavigateBack,

            Message::VaultAction(action) => Action::Run(self.handle_vault_action(action)),
            Message::VaultUpdate(update) => Action::Run(

                match update {
                    VaultUpdate::Rescan => Task::perform(
                        load_files(self.vault.clone(), self.queue_type),
                        Message::LoadFiles
                    ),
                }
            ),

        }
    }

    fn handle_file_list_action(&mut self, action: file_list::Action) -> Action {
        match action {
            file_list::Action::None               => Action::None,
            file_list::Action::OpenInObsidian(id) => self.open_obsidian(id),
        }
    }

    pub fn handle_key_event(&self, key: Key) -> Message {

        type N = keyboard::key::Named;


        match key {
            Key::Named(N::ArrowLeft)   |
            Key::Named(N::Escape)      => Message::NavigateBack,

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
            _ => self.file_list.handle_key_event(key).into()
        }

    }

    fn handle_vault_action(&mut self, action: VaultAction) -> Task<Message> {

        let tx = self.vault.clone();
        let (prop, value) = action.get_value();

        let id = self
            .file_list
            .get_selected()
            .map(|f| f.id)
        ;

        let Some(id) = id else {
            return Task::none();
        };

        Task::future(async move {

            // TODO: this doesn't actually write the changes to disk
            // i wanna get the file watch in place before then
            send_vault_cmd(&tx, SetProperty {
                id,
                prop,
                value,
            })
                .await
            ;

            Message::None
        })

    }


    fn open_obsidian(&self, id: FileId) -> Action {

        let tx = self.vault.clone();

        let cmd = OpenInObsidian {
            id,
        };

        Action::Run(
            Task::future(async move {
                send_vault_cmd(&tx, cmd).await;

                Message::None
            })
        )

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
    FileList (file_list::Message),

    LoadFiles(Vec<FileView>),
    NavigateBack,
    VaultAction(VaultAction),
    VaultUpdate(VaultUpdate),
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

impl Into<Message> for file_list::Message {
    fn into(self) -> Message {
        Message::FileList(self)
    }
}


async fn load_files(vault: Sender<VaultCommand>, queue: QueueType) -> Vec<FileView> {

    let cmd = match queue {
        QueueType::NeedsType   => |f: &MdFile| f.needs_type(),
        QueueType::NeedsAction => |f: &MdFile| f.needs_action_type(),
    };

    send_vault_cmd(
        &vault,
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
