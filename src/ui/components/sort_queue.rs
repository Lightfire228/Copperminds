use iced::Length::Fill;
use iced::keyboard;
use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Button, Column, button, column, pick_list, row, text, tooltip};

use crate::ui::{Message, QueueType, UIMode};
use crate::vault::command::VaultCommand;
use crate::vault::md_file::FileView;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortQueue {
    pub queue_type: QueueType,
    pub files:      Vec<FileView>,

}

impl SortQueue {

    pub fn view(&self) -> Element<'_, Message> {
        container(
            row![
                container(
                    column![
                        text!("{}", self.queue_type),
                        text!("==="),
                    ]
                )
                    .width(Fill)
                ,
                container(
                    column(
                        self.files.iter().map(|f|
                            text!("{}", &f.name).into()
                        )
                    ),
                )
                    .width(Fill)
            ]
            .spacing(40)
        )
        .into()
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn handle_key_event(&self, key: Key) -> Task<Message> {
        let Key::Named(key) = key else {
            return Task::none();
        };

        match key {
            keyboard::key::Named::Escape => {
                Task::future(async {
                    Message::NavigateBack
                })
            },

            _ => Task::none()
        }
    }

}

impl Into<UIMode> for SortQueue {
    fn into(self) -> UIMode {
        UIMode::SortQueue(self)
    }
}
