use iced::{Element, keyboard::Key, widget::container};
use iced::widget::{column, text};
use tokio::sync::mpsc::Sender;

use crate::ui::components::vault_stats::{self, VaultStatsComponent};
use crate::ui::key_event::KeyPressed;
use crate::ui::{self, QueueType, UIMode, send_vault_cmd};
use crate::vault::command::{GetVaultStats, NukeActionables, VaultCommand, VaultUpdate};
use crate::prelude::*;


#[derive(Debug)]
#[allow(unused)]
pub struct SelectQueue {
    vault: Sender<VaultCommand>,
    stats: VaultStatsComponent,
}

type Task = iced::Task<Message>;

#[derive(Debug)]
#[allow(unused)]
pub enum Message {
    None,
    VaultUpdate(VaultUpdate),
    VaultStats (vault_stats::Message)
}

#[derive(Debug)]
pub enum Action {
    QueueSelected(QueueType),
    Run          (Task),
}


impl SelectQueue {

    pub fn new(vault: Sender<VaultCommand>) -> (Self, Task) {
        let (stats, task) = VaultStatsComponent::new(vault.clone());

        (
            Self {
                stats,
                vault,
            },
            task.map(Message::VaultStats)
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                text!("Select queue type"),
                text!("T - Inbox"),
                text!("A - Actionables"),
                // text!("N - Nuke Actionables"),
                text!(""),
                self.stats.view().map(Message::VaultStats),
            ],
        )
            .padding(10)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        Some(match message {
            Message::None                 => None?,
            Message::VaultUpdate(message) => self.vault_update(message)?,
            Message::VaultStats (message) => self.stats.update(message)?.into(),
        })
    }

    pub fn vault_update(&mut self, message: VaultUpdate) -> Option<Action> {
        self
            .stats
            .update(vault_stats::Message::VaultUpdate(message))
            .map(|x| x.into())
    }

    pub fn handle_key_event(&mut self, key: &KeyPressed) -> Option<Action> {
        let Key::Character(key) = &key.key else {
            return None;
        };

        // match key.as_str() {
        //     "n" => {
        //         let tx = self.vault.clone();

        //         return Some(Action::Run(Task::future(async move {
        //             send_vault_cmd(&tx, NukeActionables {}).await;

        //             Message::None
        //         })))
        //     }
        //     _ => {}
        // }

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

impl From<vault_stats::Action> for Action {
    fn from(val: vault_stats::Action) -> Self {
        match val {
            vault_stats::Action::Run(task) => Action::Run(task.map(Message::VaultStats))
        }
    }
}
