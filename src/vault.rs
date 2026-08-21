
pub mod md_file;
pub mod fm;
pub mod command;

mod file_utilities;
mod watch;
mod generator;


use crate::{obsidian, vault::{command::{VaultCommand, VaultUpdate}, md_file::FileView, watch::FileData}};
use file_id::FileId;
use futures::future::join_all;
use log::{debug};
use std::{collections::HashMap, env, mem, path::{Path, PathBuf}, usize};

use tokio::{select, sync::mpsc::{self, Sender, channel}};
use walkdir::{DirEntry, WalkDir};
use trash;


use md_file::{MdFile};


pub const ENV: Env = Env::Dev;

macro_rules! regex {
    ($i:ident = $r:expr) => {
        use regex::Regex;
        use std::sync::LazyLock;

        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
    };
}

pub(crate) use regex;


#[derive(Debug)]
pub struct Index {
    md_files:    HashMap<FileId, MdFile>,

    subscribers: Vec<Sender<VaultUpdate>>,

    #[allow(unused)]
    path:        PathBuf,
}

impl Index {
    pub fn build() -> Self {
        let files    = scan_vault();

        let md_files = files
            .filter   (|f| ends_with(f, ".md"))
            .map      (|f| {
                let path = f.path().to_path_buf();
                let id   = file_id::get_file_id(&path).unwrap();

                (id, MdFile::new(FileData {
                    id,
                    name: path,
                }))
            })
            .collect()
        ;

        Self {
            md_files,
            path:        ENV.vault_path(),
            subscribers: vec![],
        }
    }

    pub fn rebuild(&mut self) {
        let subs = mem::take(&mut self.subscribers);

        *self = Index::build();

        self.subscribers = subs;
    }

    pub fn delete_empty_unnamed_files(&mut self) {

        let files: Vec<_> = self
            .iter_files()
            .filter(|f| f.is_empty() && f.is_unnamed())
            .map   (|f| f.id)
            .collect()
        ;

        for id in files.iter() {
            let path = &self.md_files[id].path;

            trash::delete(path).unwrap();
            self.md_files.remove(id);
        }
    }

    fn iter_files(&self) -> impl Iterator<Item = &MdFile> {
        self
            .md_files
            .iter()
            .map (|f| f.1)
    }

    pub fn iter_files_with<P>(&self, mut predicate: P) -> impl Iterator<Item = FileId>
    where
        P: FnMut(&MdFile) -> bool,
    {
        self
            .iter_files()
            .filter    (move |f| predicate(f))
            .map       (|f| f.id)
    }

    fn iter_files_with_cmd<P>(&self, mut predicate: P) -> Vec<FileView>
    where
        P: FnMut(&MdFile) -> bool,
    {
        self
            .iter_files()
            .filter    (|f| predicate(f))
            .map       (FileView::from)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_file(&self, id: FileId) -> &MdFile {
        &self.md_files[&id]
    }

    pub fn get_file_mut(&mut self, id: FileId) -> &mut MdFile {
        self.md_files.get_mut(&id).unwrap()
    }

    pub fn handle_command(&mut self, command: VaultCommand) {

        macro_rules! send {
            ($resp:ident => $expr:expr) => {
                _ = $resp.send($expr)
            };
        }

        match command {
            VaultCommand::IterFilesWith(filter, resp) => send!(resp =>
                self.iter_files_with_cmd(filter.filter)
            ),

            VaultCommand::SetProperty(prop, resp) => send!(resp =>
                self
                    .get_file_mut(prop.id)
                    .set_property(prop.prop, prop.value)
            ),

            VaultCommand::OpenInObsidian(opts, resp) => send!(resp => {
                let file = &self.md_files[&opts.id];

                obsidian::open_in_obsidian(file);
            }),
            VaultCommand::Register(_, resp) => send!(resp => {
                let (tx, rx) = channel(1000);

                self.subscribers.push(tx);

                rx
            })

        }
    }
}


fn scan_vault() -> impl Iterator<Item = DirEntry> {
    WalkDir::new(ENV.vault_path())
        .into_iter   ()
        .filter_entry(|e| !is_hidden(e))
        .filter_map  (|e| e.ok())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str   ()
        .map      (|f| f.starts_with("."))
        .unwrap_or(false)
}

fn ends_with(entry: &DirEntry, ext: &str) -> bool {
    entry
        .file_name()
        .to_str   ()
        .map      (|f| f.ends_with(ext))
        .unwrap_or(false)
}


pub fn serve() -> Sender<VaultCommand> {

    let (tx, rx) = mpsc::channel::<VaultCommand>(1000);

    let watcher = watch::Watcher::new(ENV.vault_path()).unwrap();

    tokio::spawn(async move {
        handle_serve(rx, watcher).await;
    });

    tx
}

async fn handle_serve(mut rx: mpsc::Receiver<VaultCommand>, mut watcher: watch::Watcher) {

    let mut index = Index::build();

    index.delete_empty_unnamed_files();

    loop {

        select! {
            command = rx.recv() => {
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

        self.rebuild();

        VaultUpdate::Rescan
    }

    fn _find_file_id_by_name(&self, file: &Path) -> FileId {
        let name = file.to_str().unwrap();

        let Some(id) = self.md_files
            .iter  ()
            .filter(|f| f.1.file_name == name)
            .map   (|f| f.0)

            .next  ()
            .copied()
        else {
            panic!("file not found: {name}");
        };

        id
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


pub fn generate_vault() {
    generator::generate_sample_vault();
}


#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Env {
    Prod,
    Dev,
}

impl Env {
    pub fn vault_name(&self) -> &'static str {
        match self {
            Self::Prod => "Notes",
            Self::Dev  => "Notes_dev",
        }
    }

    pub fn vault_path(&self) -> PathBuf {

        let home = env::home_dir().unwrap();

        home.join(self.vault_name())

    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Prod => "Prod",
            Self::Dev  => "Dev",
        }
    }
}
