use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, text};
use tokio::sync::mpsc::Sender;

use crate::ui::components::vault_stats::{self, VaultStatsComponent};
use crate::ui::key_event::KeyPressed;
use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::VaultStats;
use crate::vault::command::{GetVaultStats, VaultCommand};


#[derive(Debug)]
pub struct SelectQueue {
    stats: VaultStatsComponent,
}

type Task = iced::Task<Message>;

impl SelectQueue {

    pub fn new(vault: Sender<VaultCommand>) -> (Self, Task) {
        let (stats, task) = VaultStatsComponent::new(vault);

        (
            Self {
                stats,
            },
            task.map(Message::VaultStats)
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                text!("Select queue type"),
                text!("T - Queue"),
                text!("A - Actionables"),
                text!(""),
                self.stats.view().map(Message::VaultStats),
            ],
        )
            .padding(10)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            // Message::None => None,

            Message::VaultStats(stats) => {
                self.stats.update(stats);

                None
            },
        }
    }

    pub fn handle_key_event(&mut self, key: &KeyPressed) -> Option<Action> {
        let Key::Character(key) = &key.key else {
            return None;
        };

        let queue = match key.as_str() {
            "t" => QueueType::Inbox,
            "a" => QueueType::Actionables,
            _   => {
                return None;
            }
        };

        Some(Action::QueueSelected(queue))
    }

}

impl From<SelectQueue> for UIMode {
    fn from(val: SelectQueue) -> Self {
        UIMode::SelectQueue(val)
    }
}


#[derive(Debug)]
pub enum Message {
    // None,
    VaultStats(vault_stats::Message),
}

#[derive(Debug)]
pub enum Action {
    QueueSelected(QueueType)
}

impl From<Message> for ui::Message {
    fn from(val: Message) -> Self {
        ui::Message::SelectQueue(val)
    }
}
