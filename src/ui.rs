#![allow(unused_imports)]

use std::fmt::Display;

use iced::Length::Fill;
use iced::application::BootFn;
use iced::keyboard::{Event, Key};
use iced::wgpu::naga::proc::index;
use iced::wgpu::wgt::error;
use iced::widget::text_editor::Content;
use iced::widget::{container, row, space::horizontal, text_editor};
use iced::{Element, Font, Length, Subscription, Theme, application, event, keyboard, theme};
use iced::widget::{Button, Column, button, column, pick_list, text, tooltip};
use iced::Task;
use tokio::runtime::Runtime;
use smol_str::SmolStr;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::vault::Index;
use crate::vault::command::VaultCommand;
use crate::vault::md_file::{FileId, FileView, MdFile};


pub fn main(tx: Sender<VaultCommand>) {
    println!("iced ui");

    let starter = AppStarter {
        tx,
    };

    application(starter, App::update, App::view)
        .theme       (Theme::Dark)
        .title       ("Copperminds")
        .subscription(|_| iced::event::listen().map(Message::Event))
        .run         ()
        .unwrap      ()
    ;
}

struct App {
    index:      Sender<VaultCommand>,
    ui_mode:    UIMode,
}

#[derive(Debug, Clone)]
enum Message {
    Event        (iced::Event),
    QueueSelected(QueueType),
    QueueLoaded  (QueueType, Vec<FileView>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueueType {
    NeedsType,
    NeedsAction,
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum UIMode {
    SelectQueue,
    SortQueue(SortFileState),
}


impl App {
    fn new(tx: Sender<VaultCommand>) -> (Self, Task<Message>) {(

        Self {
            index:   tx,
            ui_mode: UIMode::SelectQueue,
        },
        Self::on_startup(),

    )}

    fn on_startup() -> Task<Message> {
        Task::none()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(iced::Event::Keyboard(event)) => {
                self.handle_key_event(event)
            }
            Message::QueueSelected(queue) => {
                Task::perform(load_files(self.index.clone(), queue), move |files| Message::QueueLoaded(queue, files))
            },
            Message::QueueLoaded(queue_type, files) => {
                self.ui_mode = UIMode::SortQueue(SortFileState {
                    queue_type,
                    files,
                });

                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn handle_key_event(&mut self, event: Event) -> Task<Message> {

        let Event::KeyPressed { key, .. } = event else {
            return Task::none();
        };

        let Key::Character(key) = key else {
            return Task::none();
        };

        let queue = match key.as_str() {
            "t" => QueueType::NeedsType,
            "a" => QueueType::NeedsAction,
            _   => {
                return Task::none();
            }
        };

        Task::future(async move {
            Message::QueueSelected(queue)
        })
    }

    fn view(&self) -> Element<'_, Message> {

        let element = match &self.ui_mode {
            UIMode::SelectQueue => {
                container(
                    column![
                        text!("Select queue type"),
                        text!("T - type"),
                        text!("A - action"),
                    ],
                )
            },
            UIMode::SortQueue(sort_queue) => {
                container(
                    column![
                        text!("{}", sort_queue.queue_type),
                        text!("==="),
                        column(
                            sort_queue.files.iter().map(|f|
                                text!("{}", &f.name).into()
                            )
                        ),
                    ]
                )
            },
        };

        element.into()

    }
}


async fn load_files(vault: Sender<VaultCommand>, queue: QueueType) -> Vec<FileView> {

    let (tx, rx) = oneshot::channel();

    let cmd = match queue {
        QueueType::NeedsType   => |f: &MdFile| f.needs_type(),
        QueueType::NeedsAction => |f: &MdFile| f.needs_action_type(),
    };

    vault
        .send(VaultCommand::IterFilesWith {
            filter: cmd,
            resp:   tx
        })
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


#[derive(Debug, Clone, PartialEq, Eq)]
struct SortFileState {
    pub queue_type: QueueType,
    pub files:      Vec<FileView>,
}



struct AppStarter {
    tx: Sender<VaultCommand>,
}

impl BootFn<App, Message> for AppStarter {
    fn boot(&self) -> (App, iced::Task<Message>) {
        App::new(self.tx.clone())
    }
}
