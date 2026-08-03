#![allow(unused_imports)]

use std::fmt::Display;

use iced::Length::Fill;
use iced::keyboard::Event;
use iced::wgpu::naga::proc::index;
use iced::wgpu::wgt::error;
use iced::widget::text_editor::Content;
use iced::widget::{container, row, space::horizontal, text_editor};
use iced::{Element, Font, Length, Subscription, Theme, application, event, keyboard, theme};
use iced::widget::{Button, Column, button, column, pick_list, text, tooltip};
use iced::Task;
use tokio::runtime::Runtime;

use crate::vault::Index;
use crate::vault::md_file::{FileId, MdFile};


pub fn main() {
    println!("iced ui");

    application(App::new, App::update, App::view)
        .theme       (Theme::Dark)
        .title       ("Copperminds")
        .subscription(App::subscription)
        .run         ()
        .unwrap      ()
    ;
}


struct App {
    index:      Index,
    file_queue: Vec<FileId>,
    queue_type: QueueType,
}

#[derive(Debug, Clone)]
enum Interaction {
    LoadQueue(QueueType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueueType {
    NeedsType,
    NeedsAction,
}



impl App {
    fn new() -> (Self, Task<Interaction>) {
        let index = Index::build();

        let queue_type = QueueType::NeedsType;
        (
            Self {
                file_queue: load_files(&index, queue_type),
                index,
                queue_type,
            },
            Task::none() // on startup

        )
    }

    fn update(&mut self, message: Interaction) -> Task<Interaction> {
        match message {
            Interaction::LoadQueue(queue) => {
                self.file_queue = load_files(&self.index, queue);
                self.queue_type = queue;
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Interaction> {

        let files = self.file_queue
            .iter()
            .map (|f| {
                let name = &self.index._get_file(*f).file_name;

                text!("{}", name).into()
            })
        ;

        container(
            column![
                self.queue_picker(),
                column(files),
            ]
        )
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Interaction> {

        event::listen_with(|event, status, _| match (event, status) {
            (iced::Event::Keyboard(event), event::Status::Ignored) => {

                Self::handle_key_event(event)

            },
            _ => None
        })
    }

    fn handle_key_event(event: iced::keyboard::Event) -> Option<Interaction> {
        match event {
            Event::KeyPressed  { key, .. } => {
                match key {
                    keyboard::Key::Character(t) if t.as_str() == "t" => Some(Interaction::LoadQueue(QueueType::NeedsType)),
                    keyboard::Key::Character(t) if t.as_str() == "a" => Some(Interaction::LoadQueue(QueueType::NeedsAction)),
                    _ => None
                }
            },
            _ => None
        }
    }

    fn queue_picker(&self) -> Element<'_, Interaction> {
        pick_list(
            [
                QueueType::NeedsType,
                QueueType::NeedsAction,
            ],
            Some(&self.queue_type),
            Interaction::LoadQueue
        )
            .into()

    }

}


fn load_files(index: &Index, queue: QueueType) -> Vec<FileId> {

    index.iter_files_with(|f| match queue {
        QueueType::NeedsType   => f.needs_type(),
        QueueType::NeedsAction => f.needs_action_type()
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
