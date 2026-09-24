use iced::{Element};
use iced::widget::{column, text};
use tokio::sync::mpsc::Sender;

use crate::ui::key_event::KeyPressed;
use crate::ui::{send_vault_cmd};
use crate::vault::VaultStats;
use crate::vault::command::{GetVaultStats, VaultCommand, VaultUpdate};


#[derive(Debug)]
pub struct VaultStatsComponent {
    stats: VaultStats,
    vault: Sender<VaultCommand>,
}

type Task = iced::Task<Message>;

impl VaultStatsComponent {

    pub fn new(vault: Sender<VaultCommand>) -> (Self, Task) {
        let tx = vault.clone();
        (
            Self {
                stats: VaultStats::default(),
                vault,
            },
            Task::future(async move {
                let stats = send_vault_cmd(&tx, GetVaultStats {}).await;

                Message::VaultStats(stats)
            })
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text!("Vault Stats"),
            text!("==="),
            text!("info        | total    - {:>5}", self.stats.info_total),
            text!("info        | archived - {:>5}", self.stats.info_archived),
            text!("info        | complete - {:>5}", self.stats.info_complete),
            text!("actionables | total    - {:>5}", self.stats.actionables_total),
            text!("actionables | open     - {:>5}", self.stats.actionables_open),
            text!("actionables | complete - {:>5}", self.stats.actionables_complete),
            text!("actionables | archived - {:>5}", self.stats.actionables_archived),
            text!(""),
            text!("needs action           - {:>5}", self.stats.needs_action),
            text!("needs sorted           - {:>5}", self.stats.needs_sorted),
            text!("illegal names          - {:>5}", self.stats.illegal_names),
            text!(""),
            text!("Open Actionables"),
            text!("==="),
            text!("todo                   - {:>5}", self.stats.open_todo),
            text!("backlog                - {:>5}", self.stats.open_backlog),
            text!("entertainment          - {:>5}", self.stats.open_entertainment),
            text!("maybe someday          - {:>5}", self.stats.open_maybe_someday),
            text!("waiting for            - {:>5}", self.stats.open_waiting_for),
            text!(""),
            text!("project files          - {:>5}", self.stats.project_files),
        ]
            .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        Some(match message {
            Message::VaultStats(stats) => {
                self.stats = stats;
                None?
            },
            Message::VaultUpdate(_) => {

                let tx  = self.vault.clone();

                Action::Run(Task::future(async move {
                    let stats = send_vault_cmd(&tx, GetVaultStats {}).await;

                    Message::VaultStats(stats)
                }))
            }
        })
    }


    pub fn handle_key_event(&mut self, _key: &KeyPressed) -> Option<Action> {
        None
    }
}


#[derive(Debug)]
pub enum Message {
    VaultStats (VaultStats),
    VaultUpdate(VaultUpdate),
}

#[derive(Debug)]
pub enum Action {
    Run(Task)
}
