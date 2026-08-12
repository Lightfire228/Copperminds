#![allow(unused_imports)]

mod components;


use std::fmt::Display;

use futures::FutureExt;
use iced::Length::Fill;
use iced::application::BootFn;
use iced::keyboard::{Event, Key};
use iced::wgpu::naga::proc::index;
use iced::wgpu::wgt::error;
use iced::widget::text_editor::Content;
use iced::widget::{container, row, space::horizontal, text_editor};
use iced::{Element, Font, Length, Subscription, Theme, application, event, keyboard, message, theme};
use iced::widget::{Button, Column, button, column, pick_list, text, tooltip};
use iced::Task;
use tokio::runtime::Runtime;
use smol_str::SmolStr;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::ui::components::select_queue::SelectQueue;
use crate::ui::components::sort_queue::{self, SortQueue, SortQueueMessage};
use crate::vault::Index;
use crate::vault::command::{Cmd, IterFilesWith, OpenInObsidian, VaultCommand};
use crate::vault::md_file::{FileId, FileView, MdFile};
use crate::obsidian;


pub fn main(tx: Sender<VaultCommand>) {
    println!("iced ui");

    let starter = AppStarter {
        tx,
    };

    application(starter, App::update, App::view)
        .theme       (Theme::Dark)
        .title       ("Copperminds")
        .subscription(|_| iced::event::listen().map(Message::Event))
        .default_font(Font::MONOSPACE)
        .run         ()
        .unwrap      ()
    ;
}

struct App {
    vault:      Sender<VaultCommand>,
    ui_mode:    UIMode,
}

#[derive(Debug)]
enum Message {
    None,
    Event           (iced::Event),
    QueueSelected   (QueueType),
    QueueLoaded     (QueueType, Vec<FileView>),
    SortQueue       (sort_queue::Message),
    OpenInObsidian  (FileId),

    #[allow(dead_code)] // Debug and Clone generate dead code for empty variants
    NavigateBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueueType {
    NeedsType,
    NeedsAction,
}


#[derive(Debug)]
enum UIMode {
    SelectQueue(SelectQueue),
    SortQueue(SortQueue),
}


impl App {
    fn new(tx: Sender<VaultCommand>) -> (Self, Task<Message>) {(

        Self {
            vault:   tx,
            ui_mode: SelectQueue::new().into(),
        },
        Self::on_startup(),

    )}

    fn on_startup() -> Task<Message> {
        Task::none()
    }

    fn update(&mut self, message: Message) -> Task<Message> {

        match message {
            Message::Event(iced::Event::Keyboard(event)) => {
                let message = self.handle_key_event(event);

                return Task::future(async move { message })
            }

            Message::QueueSelected(queue) => {
                let tx = self.vault.clone();

                return Task::future(async move {
                    Message::QueueLoaded(
                        queue,
                        load_files(tx, queue).await
                    )
                });
            },
            Message::QueueLoaded(queue_type, files) => {
                self.ui_mode = SortQueue::new(queue_type, files, self.vault.clone()).into();

                return Task::none()
            }
            Message::NavigateBack => {
                self.ui_mode = SelectQueue::new().into();

                return Task::none()
            }

            Message::OpenInObsidian(id) => {

                let tx = self.vault.clone();

                return Task::future(async move {
                    send_vault_cmd(tx, OpenInObsidian { id, }).await;

                    Message::None
                });
            }

            _ => {()},
        }

        match (&mut self.ui_mode, message) {
            (UIMode::SelectQueue(x), message) => x
                .update(message)
                .map_or_else(
                    Task::none,
                    |m| self.update(m)
                )
            ,
            (UIMode::SortQueue(x), Message::SortQueue(message)) => match x.update(message) {

                sort_queue::Action::None         => Task::none(),
                sort_queue::Action::Run(task)    => task.map(Message::SortQueue),
                sort_queue::Action::NavigateBack => self.update(Message::NavigateBack),
            }
            _ => Task::none(),
        }

    }

    fn handle_key_event(&self, event: Event) -> Message {

        let Event::KeyPressed { key, .. } = event else {
            return Message::None;
        };


        match &self.ui_mode {
            UIMode::SelectQueue(x) => x
                .handle_key_event(key)
                .unwrap_or(Message::None)
            ,
            UIMode::SortQueue  (x) => x.handle_key_event(key).into()

        }
    }

    fn view(&self) -> Element<'_, Message> {

        match &self.ui_mode {
            UIMode::SelectQueue(x) => x.view(),
            UIMode::SortQueue  (x) => x.view().map(Message::SortQueue),
        }

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

async fn send_vault_cmd<T>(vault: Sender<VaultCommand>, cmd: impl Cmd<T>) -> T {
    let (tx, rx) = oneshot::channel();

    vault
        .send(cmd.to_command(tx))
        .await
        .unwrap()
    ;

    rx.await.unwrap()
}


impl Display for QueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueType::NeedsType   => write!(f, "Needs Type"),
            QueueType::NeedsAction => write!(f, "Needs Action"),
        }
    }
}



struct AppStarter {
    tx: Sender<VaultCommand>,
}

impl BootFn<App, Message> for AppStarter {
    fn boot(&self) -> (App, iced::Task<Message>) {
        App::new(self.tx.clone())
    }
}
