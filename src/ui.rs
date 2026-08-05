#![allow(unused_imports)]

use std::fmt::Display;

use iced::Length::Fill;
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

use crate::vault::Index;
use crate::vault::md_file::{FileId, MdFile};


pub fn main() {
    println!("iced ui");

    application(App::new, App::update, App::view)
        .theme       (Theme::Dark)
        .title       ("Copperminds")
        .subscription(|_| iced::event::listen().map(Interaction::Event))
        .run         ()
        .unwrap      ()
    ;
}


struct App {
    index:      Index,
    ui_mode:    UIMode,
}

#[derive(Debug, Clone)]
enum Interaction {
    Event    (iced::Event),
    LoadQueue(QueueType),
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
    fn new() -> (Self, Task<Interaction>) {(

        Self {
            index:   Index ::build(),
            ui_mode: UIMode::SelectQueue,
        },
        Self::on_startup(),

    )}

    fn on_startup() -> Task<Interaction> {
        Task::none()
    }

    fn update(&mut self, message: Interaction) -> Task<Interaction> {
        match message {
            Interaction::Event(iced::Event::Keyboard(event)) => {
                self.handle_key_event(event)
            }
            Interaction::LoadQueue(queue) => {
                self.ui_mode = UIMode::SortQueue(SortFileState {
                    queue_type: queue,
                    files:      load_files(&self.index, queue),
                });

                Task::none()
            },

            _ => Task::none(),
        }
    }

    fn handle_key_event(&mut self, event: Event) -> Task<Interaction> {

        let Event::KeyPressed { key, .. } = event else {
            return Task::none();
        };

        let Key::Character(key) = key else {
            return Task::none();
        };

        match key.as_str() {
            "t" => self.update(Interaction::LoadQueue(QueueType::NeedsType)),
            "a" => self.update(Interaction::LoadQueue(QueueType::NeedsAction)),
            _   => Task::none()
        }
    }

    fn view(&self) -> Element<'_, Interaction> {

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


fn load_files(index: &Index, queue: QueueType) -> Vec<FileView> {

    index
        .iter_files_with(|f| match queue {
            QueueType::NeedsType   => f.needs_type(),
            QueueType::NeedsAction => f.needs_action_type()
        })
        .map(|id| FileView {
            id,
            name: index.get_file(id).file_name.clone(),
        })
        .collect()
}


impl Display for QueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueType::NeedsType   => write!(f, "Needs Type"),
            QueueType::NeedsAction => write!(f, "Needs Action"),
        }
    }
}

#[derive(Debug, Clone, Eq)]
struct FileView {
    pub id:   FileId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SortFileState {
    pub queue_type: QueueType,
    pub files:      Vec<FileView>,
}


impl PartialEq for FileView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
