
use std::fmt::Display;

use file_id::FileId;
use iced::Length::{self, Fill};
use iced::keyboard;
use tokio::fs::File;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Space, column, row, space, text};

use crate::collections::Files;
use crate::ui::components::file_list::{self, FileList};
use crate::ui::components::prompt::{self, MenuCommand, Prompt};
use crate::ui::key_event::KeyPressed;
use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::command::{Cmd, DeleteFile, IterFilesWith, ModifyFile, ModifyFileKind, OpenInObsidian, VaultCommand, VaultUpdate};
use crate::vault::fm::{FmAction, FmProperty, FmStatus, FmType, GetKey};
use crate::vault::md_file::{FileView, MdFile};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct SortQueue {
    vault:        Sender<VaultCommand>,
    queue_type:   QueueType,

    file_list:    FileList,
    prompt:       Prompt<Command>,
}


impl SortQueue {

    pub fn new(queue_type: QueueType, vault: Sender<VaultCommand>) -> (Self, Task<Message>) {
        (
            Self {
                queue_type,
                vault:     vault.clone(),
                file_list: FileList::new(),
                prompt:    Prompt  ::new(queue_type.get_command_list().to_owned()),
            },
            Task::batch([
                Task::perform(load_files(vault, queue_type), Message::LoadFiles),
            ])

        )
    }

    pub fn view(&self) -> Element<'_, Message> {

        let options = self.queue_type.get_command_list()
            .iter()
            .map (|o| text!("{} - {}", o.code, o.name).into())
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
                        text!("cursor:     {}", self.file_list.cursor()),
                        text!("file count: {}", self.file_list.file_count()),
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
    pub fn update(&mut self, message: Message) -> Option<Action> {

        Some(match message {
            Message::None => None?,

            Message::PromptMessage(message) => {
                let action = self.prompt.update(message)?;
                self.handle_prompt_action(action)?
            }

            Message::FileListMessage(message) => {
                let action = self.file_list.update(message)?;
                self.handle_file_list_action(action)?
            }
            Message::LoadFiles(files) => {
                let message = file_list::Message::LoadFiles(files.into());

                let action = self.file_list.update(message)?;
                self.handle_file_list_action(action)?
            }

            Message::VaultUpdate(update) => Action::Run(

                match update {
                    VaultUpdate::Rescan => Task::perform(
                        load_files(self.vault.clone(), self.queue_type),
                        Message::LoadFiles
                    ),
                }
            ),
        })
    }

    fn handle_file_list_action(&mut self, action: file_list::Action) -> Option<Action> {
        Some(match action {
            file_list::Action::Selected(id) => self.open_obsidian(id),
        })
    }

    fn handle_prompt_action(&mut self, action: prompt::Action<Command>) -> Option<Action> {

        Some(match action {
            prompt::Action::RunCommand(command) => Action::Run(self.handle_vault_action(command)),
            prompt::Action::OpenSelected        => {
                let id = self.file_list.get_selected()?.id;

                self.open_obsidian(id)
            }
        })
    }

    pub fn handle_key_event(&mut self, key: &KeyPressed) -> Option<Action> {

        type N = keyboard::key::Named;



        match key.key {
            Key::Named(N::ArrowLeft)   |
            Key::Named(N::Escape)      => return Some(Action::NavigateBack),

            _ => {}
        };

        if let Some(action) = self.file_list.handle_key_event(key) {
            return self.handle_file_list_action(action);
        }

        let action = self.prompt.handle_key_event(key);
        self.handle_prompt_action(action?)


    }

    fn handle_vault_action(&mut self, commands: Vec<Command>) -> Task<Message> {

        let Ok(commands) = self
            .validate_commands(commands)
            .inspect_err      (|err| warn!("{err}"))
        else {
            return Task::none();
        };

        let tx = self.vault.clone();

        let id = self
            .file_list
            .get_selected()
            .map(|f| f.id)
        ;

        let Some(id) = id else {
            return Task::none();
        };

        let is_delete = commands.iter().find(|x| matches!(x, Command::DeleteFile)).is_some();

        if is_delete {
            return Task::future(async move {
                send_vault_cmd(&tx, DeleteFile {
                    id,
                })
                    .await
                ;

                Message::PromptMessage(prompt::Message::Clear)
            })
        }

        let changes: Vec<_> = commands
            .into_iter()
            .map      (|x| x.try_into().expect("unreachable"))
            .collect  ()
        ;

        Task::future(async move {

            let res = send_vault_cmd(&tx, ModifyFile {
                id,
                changes,
            })
                .await
            ;


            match res {
                Err(err) => {
                    // TODO: make this a sort_queue message and display to the user
                    error!("Error while running vault command: {err}");
                    Message::None

                },
                Ok (_) => Message::PromptMessage(prompt::Message::Clear),
            }


        })

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

    // TODO: debounce this while holding the up/down keys
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
        QueueType::Inbox       => |f: &MdFile| f.needs_sorting(),
        QueueType::Actionables => |f: &MdFile| f.is_actionable(),
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


impl TryInto<ModifyFileKind> for Command {
    type Error = ();

    fn try_into(self) -> Result<ModifyFileKind, Self::Error> {
        Ok(match self {
            Command::SetTypeInfo           => ModifyFileKind::SetTypeInfo,
            Command::SetActionTodo         => ModifyFileKind::SetActionTodo,
            Command::SetActionWaitingFor   => ModifyFileKind::SetActionWaitingFor,
            Command::SetActionProject      => ModifyFileKind::SetActionProject,
            Command::SetActionMaybeSomeday => ModifyFileKind::SetActionMaybeSomeday,
            Command::SetStatusComplete     => ModifyFileKind::SetStatusComplete,
            Command::SetStatusArchived     => ModifyFileKind::SetStatusArchived,
            Command::DeleteFile            => Err(())?,
        })
    }
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

// TODO: make sort queue command agnostic
pub static COMMANDS: &'static [MenuCommand<Command>] = &table!(
    (SetTypeInfo,           "i", "type    - info"),
    (SetActionTodo,         "t", "action  - todo"),
    (SetActionWaitingFor,   "w", "action  - waiting for"),
    (SetActionProject,      "p", "action  - project"),
    (SetActionMaybeSomeday, "m", "action  - maybe someday"),
    (SetStatusComplete,     "c", "status  - complete"),
    (SetStatusArchived,     "a", "status  - archived"),
    (DeleteFile,            "d", "command - delete file"),
);

pub static ACTIONABLES_COMMANDS: &'static [MenuCommand<Command>] = &table!(
    // (SetTypeInfo,           "i", "type    - info"),
    // (SetActionTodo,         "t", "action  - todo"),
    // (SetActionWaitingFor,   "w", "action  - waiting for"),
    // (SetActionProject,      "p", "action  - project"),
    // (SetActionMaybeSomeday, "m", "action  - maybe someday"),
    // (SetStatusComplete,     "c", "status  - complete"),
    // (SetStatusArchived,     "a", "status  - archived"),
    // (DeleteFile,            "d", "command - delete file"),
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


impl QueueType {
    fn get_command_list(&self) -> &'static [MenuCommand<Command>] {
        match self {
            QueueType::Inbox       => COMMANDS,
            QueueType::Actionables => ACTIONABLES_COMMANDS,
        }
    }
}
