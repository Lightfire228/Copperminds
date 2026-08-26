use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, text};

use crate::ui::{self, QueueType, UIMode};


#[derive(Debug)]
pub struct SelectQueue {}

type _Task = iced::Task<Message>;

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

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::None                      => Action::None,
            Message::QueueSelected(queue_type) => Action::QueueSelected(queue_type),
        }
    }

    pub fn handle_key_event(&mut self, key: &Key) -> Action {
        let Key::Character(key) = key else {
            return Action::None;
        };

        let queue = match key.as_str() {
            "t" => QueueType::NeedsType,
            "a" => QueueType::NeedsAction,
            _   => {
                return Action::None;
            }
        };

        Action::QueueSelected(queue)
    }

}

impl From<SelectQueue> for UIMode {
    fn from(val: SelectQueue) -> Self {
        UIMode::SelectQueue(val)
    }
}


#[derive(Debug)]
pub enum Message {
    None,
    QueueSelected(QueueType)
}

#[derive(Debug)]
pub enum Action {
    None,
    QueueSelected(QueueType)
}

impl From<Message> for ui::Message {
    fn from(val: Message) -> Self {
        ui::Message::SelectQueue(val)
    }
}
