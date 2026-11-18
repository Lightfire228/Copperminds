
use crate::{config::Config, file_shit, obsidian::{self, open_in_obsidian}, vault::{Index, OpenMessage, command::{ModifyFile, ModifyFileKind, OpenInObsidian, Responder, VaultCommand, VaultUpdate}, ecs::{Ecs, EcsFileView, NewFile, components::*}, fm::*, watch}};
use file_id::FileId;
use futures::future::join_all;
use log::{debug, logger};
use std::{borrow::Cow, collections::{HashMap, HashSet}, env, mem, path::{Path, PathBuf}, sync::LazyLock, time::Duration, usize};
use crate::prelude::*;

use tokio::{select, sync::mpsc::{self, Receiver, Sender, channel}, time::{Instant, sleep}};
use walkdir::{DirEntry, WalkDir};


pub fn serve(config: &Config) -> Sender<VaultCommand> {

    let (mut index, open_rx) = Index::build(config.clone());

    // Do this before setting up the file watch
    index.delete_empty_unnamed_files();


    let (tx, rx) = mpsc::channel::<VaultCommand>(1000);

    let watcher = watch::Watcher::new(config.vault_path.clone()).unwrap();

    tokio::spawn(serve_open  (config.open_timeout, open_rx));
    tokio::spawn(handle_serve(index, rx, watcher));

    tx
}

async fn handle_serve(
    mut index:    Index,
    mut vault_rx: mpsc::Receiver<VaultCommand>,
    mut watcher:  watch::Watcher
)
    -> !
{

    loop {

        select! {
            command = vault_rx.recv() => {
                if let Some(command) = command {
                    index.handle_command(command);
                }
            }
            event = watcher.next_event() => {
                index.handle_external_fs_event(event).await;
            }
        };

    }
}

async fn serve_open(
        timeout: Duration,
    mut open_rx: mpsc::Receiver<OpenMessage>
)
    -> !
{

    loop {

        let (config, mut name) = open_rx.recv().await.expect("debounce borken");

        loop {
            select! {
                _ = sleep(timeout) => {
                    break;
                }
                Some((_, n)) = open_rx.recv() => {
                    name = n;
                }
            };
        }

        open_in_obsidian(&config, &name);

    }
}



impl Index {
    async fn handle_external_fs_event(&mut self, event: Option<watch::ModificationType>) {
        let Some(event) = event else {
            return;
        };

        debug!("Handle event {event:?}");

        // MAYBE: when i hit `ctrl + s` in obsidian, I get 2 update events
        //        do i need to "debounce" events by a few millis?

        type Mt = watch::ModificationType;
        let event = match event {
            Mt::Unknown => self.handle_external_unknown_event(),
        };

        self.send_notifications(event).await;
    }


    fn handle_external_unknown_event(&mut self) -> VaultUpdate {
        debug!("Rebuilding the index");

        self.rebuild_index();

        VaultUpdate::Rescan
    }

    async fn send_notifications(&mut self, event: VaultUpdate) {

        let futures = self.subscribers
            .iter()
            .map(|s| async move {
                s.send(event).await
            })
        ;

        let res = join_all(futures).await;

        let closed = res
            .iter      ()
            .enumerate ()
            .filter_map(|s| s.1.map_err(|_| s.0).err())
        ;

        let mut prv = usize::MAX;

        for s in closed.rev() {

            assert!(s < prv, "the subscriber indices got all fucked");
            prv = s;

            self.subscribers.remove(s);

        }
    }
}
