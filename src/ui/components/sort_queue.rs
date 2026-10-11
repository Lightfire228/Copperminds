
use std::fmt::Display;

use file_id::FileId;
use iced::Length::{Fill};
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Space, column, row, text};

use crate::collections::Files;
use crate::ui::actions::UiAction;
use crate::ui::components::file_list::{self, FileList};
use crate::ui::components::prompt::{self, MenuCommand, Prompt};
use crate::ui::key_event::KeyPressed;
use crate::ui::filter::{UiFilter, FilterByList};
use crate::ui::{self, UIMode, send_vault_cmd};
use crate::ui::queue_type::QueueType;
use crate::vault::command::{DeleteFile, IterFilesWith, ModifyFile, ModifyFileKind, OpenInObsidian, VaultCommand, VaultUpdate};
use crate::vault::ecs::EcsFileView;
use crate::vault::fm::*;

use crate::prelude::*;

pub struct SortQueue {
    vault:        Sender<VaultCommand>,
    queue_type:   QueueType,

    file_list:    FileList,
    prompt:       Prompt<UiAction>,
    command_list: Vec<MenuCommand<UiAction>>
}




impl SortQueue {

    pub fn new(queue_type: QueueType, vault: Sender<VaultCommand>) -> (Self, Task<Message>) {

        let command_list: Vec<_> = UiAction::all()
            .into_iter()
            .filter_map   (|c| Some(MenuCommand {
                code:    c.get_code(queue_type)?,
                name:    c.get_label(),
                command: c,
            }))
            .collect()
        ;

        (
            Self {
                vault:     vault.clone(),
                file_list: FileList::new(),
                prompt:    Prompt  ::new(command_list.iter().copied()),
                queue_type,
                command_list,
            },
            Task::batch([
                Task::perform(load_files(vault, queue_type), Message::LoadFiles),
            ])

        )
    }

    pub fn view(&self) -> Element<'_, Message> {

        let options = self.command_list
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
                        text!("index:      {}", self.file_list.index()),
                        text!("cursor:     {}", self.file_list.cursor()),
                        text!("scroll:     {}", self.file_list.scroll()),
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

    fn handle_prompt_action(&mut self, action: prompt::Action<UiAction>) -> Option<Action> {

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

    fn handle_vault_action(&mut self, commands: Vec<UiAction>) -> Task<Message> {

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

        let is_delete = commands.iter().find(|x| matches!(x, UiAction::DeleteFile)).is_some();

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

    fn validate_commands(&self, commands: Vec<UiAction>) -> Result<Vec<UiAction>, String> {

        let mut delete     = vec![];
        let mut not_delete = vec![];

        for cmd in commands.iter() {
            match cmd {
                UiAction::DeleteFile => delete    .push(cmd),
                _                    => not_delete.push(cmd),
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

    let filter = match queue {

        QueueType::Inbox       => UiFilter {
            needs_sorted: true.into(),

            ..Default::default()
        },

        QueueType::Actionables => UiFilter {
            by_action: FilterByList::Include(vec![FmAction::Todo]),
            is_open:   true.into(),

            ..Default::default()
        },
    };

    send_vault_cmd(
        &vault,
        IterFilesWith {
            filter: Box::new(move |f: &EcsFileView| filter.matches(f)),
        }
    )
    .await
    .into()
}


impl TryInto<ModifyFileKind> for UiAction {
    type Error = ();

    fn try_into(self) -> Result<ModifyFileKind, Self::Error> {
        Ok(match self {
            UiAction::SetTypeInfo  => ModifyFileKind::SetTypeInfo,
            UiAction::SetAction(a) => ModifyFileKind::SetAction(a),
            UiAction::SetStatus(s) => ModifyFileKind::SetStatus(s),
            _                      => Err(())?,
        })
    }
}
