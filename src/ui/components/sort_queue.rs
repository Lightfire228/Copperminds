use file_id::FileId;
use iced::Length::{self, Fill};
use iced::keyboard;
use tokio::fs::File;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Space, column, row, space, text};

use crate::collections::Files;
use crate::ui::components::file_list::{self, FileList};
use crate::ui::components::prompt::{self, Prompt};
use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::command::{IterFilesWith, OpenInObsidian, SetProperty, VaultCommand, VaultUpdate};
use crate::vault::fm::{FmAction, FmProperty, FmStatus, FmType, GetKey};
use crate::vault::md_file::{FileView, MdFile};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct SortQueue {
    vault:        Sender<VaultCommand>,
    queue_type:   QueueType,

    file_list:    FileList,
    prompt:       Prompt,
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
                vault:     vault.clone(),
                file_list: FileList::new(),
                prompt:    Prompt  ::new(),
            },
            Task::batch([
                Task::perform(load_files(vault, queue_type), Message::LoadFiles),
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

                        Space::new().height(Fill),

                        text!("==="),
                        self.prompt.view().map(|_| Message::None)

                    ]
                )
                    .width  (300)
                    .padding(10)
                ,
                self.file_list.view().map(Message::FileListMessage),
            ]
            .spacing(40)
        )
        .into()
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {

        match message {
            Message::None => Action::None,

            Message::FileListMessage(message) => {
                let action = self.file_list.update(message);
                self.handle_file_list_action(action)
            }
            Message::PromptMessage(message) => {
                let action = self.prompt.update(message);
                self.handle_prompt_action(action)
            }
            Message::LoadFiles(files) => {
                let message = file_list::Message::LoadFiles(files.into());

                let action = self.file_list.update(message);
                self.handle_file_list_action(action)
            }

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
            file_list::Action::None         => Action::None,
            file_list::Action::Selected(id) => self.open_obsidian(id),
        }
    }

    fn handle_prompt_action(&mut self, action: prompt::Action) -> Action {
        match action {
            prompt::Action::None => Action::None,
        }
    }

    pub fn handle_key_event(&mut self, key: Key) -> Action {

        type N = keyboard::key::Named;

        match key {
            Key::Named(N::ArrowLeft)   |
            Key::Named(N::Escape)      => return Action::NavigateBack,

            Key::Character(ref key) => {
                let list = match self.queue_type {
                    QueueType::NeedsType   => NEEDS_TYPE,
                    QueueType::NeedsAction => NEEDS_ACTION,
                };

                let action = list.iter().filter(|a| a.key == key.as_str()).next();

                if let Some(action) = action {
                    return Action::Run(self.handle_vault_action(action.action))
                };

            },
            _ => {}
        };

        match self.file_list.handle_key_event(&key) {
            file_list::Action::None => {},

            action => return self.handle_file_list_action(action),
        };

        let action = self.prompt.handle_key_event(key);
        self.handle_prompt_action(action)


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
            send_vault_cmd(&tx, SetProperty {
                id,
                prop,
                value,
            })
                .await
        })
            .discard()

    }


    fn open_obsidian(&self, id: FileId) -> Action {

        let tx = self.vault.clone();

        let cmd = OpenInObsidian {
            id,
        };

        let future = async move {
            send_vault_cmd(&tx, cmd).await;
        };

        Action::Run(
            Task::future(future).discard()
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
    FileListMessage (file_list::Message),
    PromptMessage   (prompt   ::Message),

    LoadFiles(Files),
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


async fn load_files(vault: Sender<VaultCommand>, queue: QueueType) -> Files {

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
    .into()
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
