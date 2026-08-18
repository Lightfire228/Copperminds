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

use crate::ui::components::select_queue::{self, SelectQueue};
use crate::ui::components::sort_queue::{self, SortQueue};
use crate::vault::{self, ENV, Env, Index};
use crate::vault::command::{Cmd, IterFilesWith, OpenInObsidian, VaultCommand};
use crate::vault::md_file::{FileView, MdFile};
use crate::obsidian;


pub fn main(tx: Sender<VaultCommand>) {
    println!("iced ui");

    let starter = AppStarter {
        tx,
    };

    application(starter, App::update, App::view)
        .theme       (Theme::Dark)
        .title       (App  ::title)
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
    SelectQueue     (select_queue::Message),
    SortQueue       (sort_queue  ::Message),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueueType {
    NeedsType,
    NeedsAction,
}


#[derive(Debug)]
enum UIMode {
    SelectQueue(SelectQueue),
    SortQueue  (SortQueue),
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

    fn title(&self) -> String {
        format!("Copperminds - {}", ENV.name())
    }

    fn update(&mut self, message: Message) -> Task<Message> {

        match (&mut self.ui_mode, message) {
            (_, Message::Event(iced::Event::Keyboard(event))) => {
                let message = self.handle_key_event(event);

                return Task::future(async move { message })
            }

            (UIMode::SelectQueue(x), Message::SelectQueue(message)) => match x.update(message) {
                select_queue::Action::None                      => Task::none(),
                select_queue::Action::QueueSelected(queue_type) => {
                    let (state, task) = SortQueue::new(queue_type, self.vault.clone());

                    self.ui_mode = state.into();

                    task.map(Message::SortQueue)

                }
            }
            (UIMode::SortQueue(x), Message::SortQueue(message)) => match x.update(message) {

                sort_queue::Action::None         => Task::none (),
                sort_queue::Action::Run(task)    => task.map   (Message::SortQueue),
                sort_queue::Action::NavigateBack => {
                    self.ui_mode = SelectQueue::new().into();
                    Task::none()
                },
            }
            _ => Task::none(),
        }

    }

    fn handle_key_event(&self, event: Event) -> Message {

        let Event::KeyPressed { key, .. } = event else {
            return Message::None;
        };


        match &self.ui_mode {
            UIMode::SelectQueue(x) => x.handle_key_event(key).into(),
            UIMode::SortQueue  (x) => x.handle_key_event(key).into(),
        }
    }

    fn view(&self) -> Element<'_, Message> {

        match &self.ui_mode {
            UIMode::SelectQueue(x) => x.view().map(Message::SelectQueue),
            UIMode::SortQueue  (x) => x.view().map(Message::SortQueue),
        }

    }
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
