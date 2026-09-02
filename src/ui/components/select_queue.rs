use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, text};
use tokio::sync::mpsc::Sender;

use crate::ui::key_event::KeyPressed;
use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::VaultStats;
use crate::vault::command::{GetVaultStats, VaultCommand};


#[derive(Debug)]
pub struct SelectQueue {
    stats: VaultStats,
}

type Task = iced::Task<Message>;

impl SelectQueue {

    pub fn new(vault: Sender<VaultCommand>) -> (Self, Task) {
        (
            Self {
                stats: VaultStats::default(),
            },
            Task::future(async move {
                let stats = send_vault_cmd(&vault, GetVaultStats {}).await;

                Message::VaultStats(stats)
            })
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                text!("Select queue type"),
                text!("T - Queue"),
                text!("A - Actionables"),
                text!(""),
                text!("Vault Stats"),
                text!("==="),
                text!("info        | total    - {}", self.stats.info_total),
                text!("info        | archived - {}", self.stats.info_archived),
                text!("info        | complete - {}", self.stats.info_complete),
                text!("actionables | total    - {}", self.stats.actionables_total),
                text!("actionables | open     - {}", self.stats.actionables_open),
                text!("actionables | complete - {}", self.stats.actionables_complete),
                text!("actionables | archived - {}", self.stats.actionables_archived),
                text!("needs action           - {}", self.stats.needs_action),
            ],
        )
            .padding(10)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            // Message::None => None,

            Message::VaultStats(stats) => {
                self.stats = stats;

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
    VaultStats(VaultStats),
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
