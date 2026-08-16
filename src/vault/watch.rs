
use std::{panic, path::PathBuf};

use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher as INWatcher};



#[derive(Debug)]
pub enum ModificationType {
    Create { target: PathBuf },
    Update { target: PathBuf },
    Rename { from:   PathBuf, to: PathBuf },
    Delete { target: PathBuf },
}

pub struct Watcher {
    #[allow(dead_code)]
    watcher: INotifyWatcher,

    rx:      Receiver<notify::Result<Event>>,
}

impl Watcher {

    // https://github.com/notify-rs/notify/blob/main/examples/async_monitor.rs
    pub fn new(folder: PathBuf) -> notify::Result<Watcher> {

        let (mut tx, rx) = channel(1);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                futures::executor::block_on(async {
                    tx.send(res).await.unwrap();
                })
            },
            Config::default(),
        )?;

        watcher
            .watch(folder.as_ref(), RecursiveMode::Recursive)
            .expect("Unable to add watch folder")
        ;

        Ok(Watcher {
            watcher,
            rx,
        })
    }

    pub async fn next_event(&mut self) -> Option<ModificationType> {

        while let Some(event) = self.rx.next().await {

            let res = self.handle_event(event.unwrap()).await;

            if res.is_none() {
                continue;
            }

            return res
        }

        None

    }

    async fn handle_event(&self, event: Event) -> Option<ModificationType> {


        let is_git = event
            .paths
            .iter()
            .any(|p| p
                .ancestors()
                .any(|p| p.is_dir() && p.ends_with(".git"))
            )
        ;

        let is_obsidian = event
            .paths
            .iter()
            .any(|p| p
                .ancestors()
                .any(|p| p.is_dir() && p.ends_with(".obsidian"))
            )
        ;

        if is_git || is_obsidian {
            return None;
        }

        fn take(event: Event) -> PathBuf {
            event.paths.into_iter().nth(0).unwrap()
        }

        fn rename(event: Event) -> ModificationType {
            let mut iter = event.paths.into_iter().take(2);

            ModificationType::Rename {
                from: iter.next().unwrap(),
                to:   iter.next().unwrap(),
            }
        }

        type Ek = notify::EventKind;
        type Ck = notify::event::CreateKind;
        type Mk = notify::event::ModifyKind;
        type Rm = notify::event::RenameMode;
        type Rk = notify::event::RemoveKind;


        Some(match event.kind {

            Ek::Create(Ck::File)           => ModificationType::Create { target: take(event) },
            Ek::Modify(Mk::Data(_))        => ModificationType::Update { target: take(event) },

            Ek::Modify(Mk::Name(Rm::Both)) => rename(event),

            Ek::Remove(Rk::File)           => ModificationType::Delete { target: take(event) },

            // TODO: are these relevant?
            Ek::Modify(Mk::Name(Rm::To))   => panic!("rename mode not supported"),
            Ek::Modify(Mk::Name(Rm::From)) => panic!("rename mode not supported"),

            _ => None?,
        })
    }
}
