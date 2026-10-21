
use std::collections::HashSet;
use std::fmt::Display;

use file_id::FileId;
use iced::Length::{Fill};
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Space, column, row, text};

use crate::collections::Files;
use crate::command_parser::{CommandExpr, CommandGroup};
use crate::ui::actions::UiAction;
use crate::ui::components::file_list::{self, FileList};
use crate::ui::components::prompt::{self, Prompt};
use crate::ui::key_event::KeyPressed;
use crate::ui::filter::{UiFilter, FilterByList};
use crate::ui::{self, UIMode, send_vault_cmd};
use crate::ui::queue_type::QueueType;
use crate::vault::command::{DeleteFile, GetProjects, IterFilesWith, ModifyFile, ModifyFileKind, OpenInObsidian, VaultCommand, VaultUpdate};
use crate::vault::ecs::EcsFileView;
use crate::vault::fm::*;

use crate::prelude::*;

pub struct SortQueue {
    vault:        Sender<VaultCommand>,
    queue_type:   QueueType,

    file_list:    FileList,
    prompt:       Prompt,
    projects:     Vec<String>,
    // command_list: Vec<MenuCommand<UiAction>>
}




impl SortQueue {

    pub fn new(queue_type: QueueType, vault: Sender<VaultCommand>) -> (Self, Task<Message>) {

        (
            Self {
                vault:     vault.clone(),
                file_list: FileList::new(),
                prompt:    Prompt  ::new(),
                projects:  Vec     ::new(),
                queue_type,
            },
            Task::batch([
                Task::perform(load_files   (vault.clone(), queue_type), Message::LoadFiles),
                Task::perform(load_projects(vault),                     Message::LoadProjects),
            ])

        )
    }

    pub fn view(&self) -> Element<'_, Message> {

        // TODO: aliases for actions and status
        //
        // TODO: split this into a "help" component and a "projects list" component

        let options = [
            text!(":i                - set info"),
            text!(""),
            text!(":a todo           - set action"),
            text!(":a backlog        -"),
            text!(":a maybe_someday  -"),
            text!(":a waiting_for    -"),
            text!(""),
            text!(":s complete       - set status"),
            text!(":s archive        -"),
            text!(""),
            text!(":p [project name] -"),
            text!(""),
            text!(":d                - delete file"),
        ]
            .into_iter()
            .map(|x| x.into())
        ;

        let projects = self
            .projects
            .iter()
            .map(|p| text!("- {p}").into())
        ;

        container(
            row![
                container(
                    column![
                        text!("{}", self.queue_type),
                        text!("==="),
                        column(options),
                        text!(""),

                        text!("Projects"),
                        text!("==="),
                        column(projects),

                        Space::new().height(Fill),

                        text!("==="),
                        text!("index:      {}", self.file_list.index()),
                        text!("cursor:     {}", self.file_list.cursor()),
                        text!("scroll:     {}", self.file_list.scroll()),
                        text!("file count: {}", self.file_list.file_count()),
                        self.prompt.view().map(|_| Message::None)

                    ]
                )
                    .width  (350)
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

            Message::LoadProjects(projects) => {
                self.projects = projects.into_iter().collect();

                self.projects.sort();

                None?
            }

            Message::VaultUpdate(update) => Action::Run(

                match update {
                    VaultUpdate::Rescan => self.reload_task()
                }
            ),
        })
    }

    fn reload_task(&self) -> Task<Message> {

        let vault = || self.vault.clone();

        Task::batch([
            Task::perform(load_files   (vault(), self.queue_type), Message::LoadFiles),
            Task::perform(load_projects(vault()),                  Message::LoadProjects),
        ])

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

    fn handle_vault_action(&mut self, commands: Vec<CommandExpr>) -> Task<Message> {

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
            .map      (|x| x.try_into().expect("not a valid modify action"))
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

    fn validate_commands(&self, commands: Vec<CommandExpr>) -> Result<Vec<UiAction>, String> {

        let commands: Vec<UiAction> = commands
            .into_iter()
            .map      (|x| x.into())
            .collect  ()
        ;

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

    LoadFiles   (Files),
    LoadProjects(HashSet<String>),
    VaultUpdate (VaultUpdate),
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

            // TODO: make this configurable from the UI
            by_action: FilterByList::Include(vec![
                FmAction::Todo,
                FmAction::Backlog,
                // FmAction::MaybeSomeday,
                // FmAction::WaitingFor,

            ]),
            has_project: false.into(),
            by_status:   FilterByList::Exclude(vec![
                FmStatus::Archived,
            ]),
            // is_open:   true.into(),


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

async fn load_projects(vault: Sender<VaultCommand>) -> HashSet<String> {
    send_vault_cmd(
        &vault,
        GetProjects {}
    )
    .await
    .into()
}


impl TryInto<ModifyFileKind> for UiAction {
    type Error = ();

    fn try_into(self) -> Result<ModifyFileKind, Self::Error> {
        Ok(match self {
            UiAction::SetTypeInfo   => ModifyFileKind::SetTypeInfo,
            UiAction::SetAction (a) => ModifyFileKind::SetAction (a),
            UiAction::SetStatus (s) => ModifyFileKind::SetStatus (s),
            UiAction::SetProject(p) => ModifyFileKind::SetProject(p),
            _                       => Err(())?,
        })
    }
}
