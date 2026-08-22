use futures::{never::Never};
use iced::{task::{Sipper, sipper}};
use tokio::sync::{mpsc::{Sender}};

use crate::prelude::*;

use crate::{ui::send_vault_cmd, vault::command::{Register, VaultCommand, VaultUpdate}};


pub fn connect(vault: Sender<VaultCommand>) -> impl Sipper<Never, VaultUpdate> {

    // MAYBE: is a sipper required?
    // the example uses it https://github.com/iced-rs/iced/blob/0.14/examples/websocket/src/echo.rs
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
