use futures::{SinkExt, Stream, never::Never};
use iced::{stream, task::{Sipper, sipper}};
use log::info;
use tokio::sync::{mpsc::{self, Sender, Receiver}};

use crate::prelude::*;

use crate::{ui::send_vault_cmd, vault::command::{Register, VaultCommand, VaultUpdate}};


pub fn connect(vault: Sender<VaultCommand>) -> impl Sipper<Never, VaultUpdate> {
    sipper(async move |mut output| {
        debug!("running vault subscription");

        let mut rx = send_vault_cmd(&vault, Register {}).await;

        while let Some(event) = rx.recv().await {

            _ = output.send(event).await;

        }

        error!("vault sub has closed");
        panic!();

    })
}
