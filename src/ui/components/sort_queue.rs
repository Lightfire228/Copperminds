
use file_id::FileId;
use iced::Length::{self, Fill};
use iced::keyboard;
use tokio::fs::File;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Space, column, row, space, text};

use crate::collections::Files;
use crate::ui::components::file_list::{self, FileList};
use crate::ui::components::prompt::{self, COMMANDS, Command, Prompt};
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
    prompt:       Prompt,
}


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

        let options = COMMANDS
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

    fn handle_prompt_action(&mut self, action: prompt::Action) -> Option<Action> {

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

    fn handle_vault_action(&mut self, commands: Vec<prompt::Command>) -> Task<Message> {

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
            .map(|x| match x {
                Command::SetTypeInfo           => ModifyFileKind::SetTypeInfo,
                Command::SetActionTodo         => ModifyFileKind::SetActionTodo,
                Command::SetActionWaitingFor   => ModifyFileKind::SetActionWaitingFor,
                Command::SetActionProject      => ModifyFileKind::SetActionProject,
                Command::SetActionMaybeSomeday => ModifyFileKind::SetActionMaybeSomeday,
                Command::SetStatusComplete     => ModifyFileKind::SetStatusComplete,
                Command::SetStatusArchived     => ModifyFileKind::SetStatusArchived,
                Command::DeleteFile            => unreachable!(),
            })
            .collect()
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
