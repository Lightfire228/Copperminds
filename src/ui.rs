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
    index: Index
}

enum Interaction {

}

impl App {
    fn new() -> (Self, Task<Interaction>) {(
        Self {
            index: Index::build(),
        },
        Task::none() // on startup

    )}

    fn update(&mut self, message: Interaction) -> Task<Interaction> {
        match message {

        }

        // Task::none()
    }

    fn view(&self) -> Element<'_, Interaction> {

        let files = self
            .index
            .iter_files_with(|f| f.is_untyped())
            .map            (|f| {
                let name = &self.index._get_file(f).file_name;

                text!("{}", name).into()
            })
        ;

        column(files).into()
    }


}
