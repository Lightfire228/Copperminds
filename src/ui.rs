#![allow(unused_imports)]

mod components;
mod vault_subscription;


use std::fmt::Display;
use std::hash::Hash;

use futures::{FutureExt, StreamExt};
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
use pretty_env_logger::formatted_builder;
use tokio::runtime::Runtime;
use smol_str::SmolStr;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;

use crate::ui::components::select_queue::{self, SelectQueue};
use crate::ui::components::sort_queue::{self, SortQueue};
use crate::vault::{self, ENV, Env, Index};
use crate::vault::command::{Cmd, IterFilesWith, OpenInObsidian, VaultCommand, VaultUpdate};
use crate::vault::md_file::{FileView, MdFile};
use crate::obsidian;
use crate::prelude::*;

use std::mem;


pub fn main(tx: Sender<VaultCommand>) {
    info!("iced ui");

    debug!("size of Message: {}", mem::size_of::<Message>());



    application(
        move || App::new(tx.clone()),
        App::update,
        App::view
    )
        .theme       (Theme::Dark)
        .title       (App  ::title)
        .subscription(App::subscription)
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
    VaultUpdate     (VaultUpdate)
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

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::event::listen().map(Message::Event),
            {

                let vault = VaultSubscriber { sub: self.vault.clone() };

                Subscription::run_with(vault, |vault| vault_subscription::connect(vault.sub.clone())).map(Message::VaultUpdate)
            }
        ])

    }

    fn update(&mut self, message: Message) -> Task<Message> {

        match (&mut self.ui_mode, message) {
            (_, Message::Event(iced::Event::Keyboard(event))) => {
                let message = self.handle_key_event(event);

                return Task::done(message)
            }
            (_, Message::VaultUpdate(message)) => {
                debug!("Recevied vault update {message:?}");

                // TODO: maybe restructure this
                self.update(match &self.ui_mode {
                    UIMode::SelectQueue(_) => Message::None,
                    UIMode::SortQueue  (_) => Message::SortQueue(sort_queue::Message::VaultUpdate(message)),
                })
            },

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



async fn send_vault_cmd<T>(vault: &Sender<VaultCommand>, cmd: impl Cmd<T>) -> T {
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


struct VaultSubscriber {
    sub: Sender<VaultCommand>,
}

// Subscribers are identified by their data's hash, and their function pointer,
// but i only need one vault subscriber per app
// 
// this may cause weirdness depending on what iced does with the subscribers on repeated app inits
impl Hash for VaultSubscriber {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}
