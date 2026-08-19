
use std::path::{Path, PathBuf};

use file_id::FileId;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher as INWatcher, event::EventKindMask};
use tokio::io;
use crate::prelude::*;


#[derive(Debug)]
pub enum ModificationType {
    Create (FileData),
    Update (FileData),
    Rename (FileData, PathBuf),
    Delete (FileData),
    Unknown(FileData),
}

#[derive(Debug, Clone)]
pub struct FileData {
    pub id:   FileId,
    pub name: PathBuf,
}

pub struct Watcher {
    #[allow(dead_code)]
    watcher: INotifyWatcher,

    rx:      Receiver<notify::Result<Event>>,
    count:   usize,
}

impl Watcher {

    // https://github.com/notify-rs/notify/blob/main/examples/async_monitor.rs
    pub fn new(folder: PathBuf) -> notify::Result<Watcher> {

        let (mut tx, rx) = channel(1);

        let config = Config::default().with_event_kinds(EventKindMask::CORE);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                futures::executor::block_on(async {
                    tx.send(res).await.unwrap();
                })
            },
            config,
        )?;

        watcher
            .watch(folder.as_ref(), RecursiveMode::Recursive)
            .expect("Unable to add watch folder")
        ;

        Ok(Watcher {
            watcher,
            rx,
            count: 0,
        })
    }

    pub async fn next_event(&mut self) -> Option<ModificationType> {

        while let Some(event) = self.rx.next().await {
            self.count += 1;

            let res = self.handle_event(event.unwrap()).await;

            if res.is_none() {
                continue;
            }

            return res
        }

        None

    }

    async fn handle_event(&self, event: Event) -> Option<ModificationType> {

        // TODO: unify filter logic into one place for both initial vault indexing, and events
        //       - is '.md'
        //       - is hidden
        //       - is '.git'
        //       - is '.obsidian'

        // TODO: should probably exclude vault path prefix from filter checking logic
        //       ex. /home/user/.cache/moon_logic/vault/...
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

        let is_hidden = event
            .paths
            .iter()
            .any(|p| p.file_name().unwrap().to_str().unwrap().starts_with("."))
        ;

        let filters = [
            is_git,
            is_obsidian,
            is_hidden,
        ];

        if filters.iter().any(|f| *f) {
            return None;
        }

        let mut iter   = event.paths.into_iter();
        let     name_0 = iter.next()?;
        let     name_1 = iter.next();

        let id = get_id(name_0.as_ref()).ok()?;

        let data = FileData {
            id,
            name: name_0,
        };

        type Ek = notify::EventKind;
        type Ck = notify::event::CreateKind;
        type Mk = notify::event::ModifyKind;
        type Rm = notify::event::RenameMode;
        type Rk = notify::event::RemoveKind;

        Some(match event.kind {

            Ek::Create(Ck::File)             => ModificationType::Create(data),
            Ek::Modify(Mk::Data(_))          => ModificationType::Update(data),
            Ek::Remove(Rk::File)             => ModificationType::Delete(data),
            Ek::Modify(Mk::Name(Rm::Both))   => {
                let name_1 = name_1.unwrap();
                let id     = get_id(&name_1).ok()?;

                ModificationType::Rename(
                    FileData {
                        id,
                        name: name_1,
                    },
                    data.name,
                )
            },

            Ek::Access(_) => None?,

            _ => {
                let count = self.count;
                let kind = event.kind;

                warn!("[{count}] Unknown file watch: {kind:?}, {:?}", data.name);

                ModificationType::Unknown(data)
            },
        })
    }
}

fn get_id(name: &Path) -> io::Result<FileId> {
    file_id::get_file_id(name)
        .inspect_err(|_| error!("unable to get file id for name: {name:?}"))
}
