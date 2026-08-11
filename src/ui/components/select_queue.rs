use tokio::sync::mpsc::Sender;
use iced::{Element, Task, keyboard::Key, widget::container};
use iced::widget::{Button, Column, button, column, pick_list, text, tooltip};

use crate::ui::{Message, QueueType, UIMode};
use crate::vault::command::VaultCommand;


#[derive(Debug)]
pub struct SelectQueue {}

impl SelectQueue {

    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                text!("Select queue type"),
                text!("T - type"),
                text!("A - action"),
            ],
        )
            .padding(10)
            .into()
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn handle_key_event(&self, key: Key) -> Task<Message> {
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

}

impl From<SelectQueue> for UIMode {
    fn from(val: SelectQueue) -> Self {
        UIMode::SelectQueue(val)
    }
}
