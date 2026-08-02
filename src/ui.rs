#![allow(unused_imports)]

use iced::Length::Fill;
use iced::wgpu::naga::proc::index;
use iced::wgpu::wgt::error;
use iced::widget::text_editor::Content;
use iced::widget::{container, row, space::horizontal, text_editor};
use iced::{Element, Font, Length, Subscription, Theme, application, keyboard, theme};
use iced::widget::{Button, Column, button, column, pick_list, text, tooltip};
use iced::Task;
use tokio::runtime::Runtime;

use crate::vault::Index;
use crate::vault::md_file::{FileId, MdFile};


pub fn main() {
    println!("iced ui");

    application(App::new, App::update, App::view)
        .theme (Theme::Dark)
        .title ("Copperminds")
        .run   ()
        .unwrap()
    ;
}


struct App {
    index: Index,
    files: IterFiles,
}

enum Interaction {
    NextFile,
}



impl App {
    fn new() -> (Self, Task<Interaction>) {
        let index = Index::build();

        (
            Self {
                files: load_files(&index, QueueType::NeedsAction),
                index,
            },
            Task::none() // on startup

        )
    }

    fn update(&mut self, message: Interaction) -> Task<Interaction> {
        match message {
            Interaction::NextFile => {
                self.files.next();
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Interaction> {

        let files = self.files.files
            .iter()
            .map (|f| {
                let name = &self.index._get_file(*f).file_name;

                text!("{}", name).into()
            })
        ;

        column(files).into()
    }

}


struct IterFiles {
    files: Vec<FileId>,
    queue: QueueType,
    index: usize,
}

impl Iterator for IterFiles {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {

        self.index += 1;

        self.files.get(self.index -1).copied()
    }
}


enum QueueType {
    NeedsType,
    NeedsAction,
}

fn load_files(index: &Index, queue: QueueType) -> IterFiles {

    let files: Vec<_> = index.iter_files_with(|f| match queue {
        QueueType::NeedsType   => f.needs_type(),
        QueueType::NeedsAction => f.needs_action_type()
    })
        .collect()
    ;

    IterFiles {
        files,
        queue,
        index: 0,
    }
}
