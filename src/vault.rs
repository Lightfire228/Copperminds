
pub mod md_file;
pub mod fm;
pub mod command;
pub mod ecs;

mod file_utilities;
mod watch;
mod generator;


use crate::{file_shit, obsidian, vault::{command::{ModifyFile, ModifyFileKind, OpenInObsidian, VaultCommand, VaultUpdate}, ecs::{ComponentKind, Ecs, NewFile}, fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}, md_file::FileView, watch::FileData}};
use file_id::FileId;
use futures::future::join_all;
use log::{debug};
use std::{collections::HashMap, env, mem, path::{Path, PathBuf}, usize};
use crate::prelude::*;

use tokio::{select, sync::{mpsc::{self, Sender, Receiver, channel}}};
use walkdir::{DirEntry, WalkDir};


use trash;


pub use ecs::FileView as EcsFileView;

pub const ENV: Env = Env::Dev;

macro_rules! build_regex {
    ($i:ident = $r:expr) => {
        use regex::Regex;
        use std::sync::LazyLock;

        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
    };
}

pub(crate) use build_regex;


// TODO: restructure this like an ECS
pub struct Index {
    subscribers: Vec<Sender<VaultUpdate>>,

    _path:       PathBuf,

    ecs: Ecs,
}

impl Index {
    pub fn build() -> Self {
        let mut ecs = Ecs::new();

        let files = scan_vault()
            .filter   (|f| ends_with(f, ".md"))
        ;

        for file in files {
            let path = file.path().to_path_buf();
            let id   = file_id::get_file_id(&path).unwrap();

            ecs.new_file(NewFile {
                id,
                path:     path.clone(),
                raw_text: file_shit::get_file_text(&path),
                name:     file_shit::get_file_name(&path),
            });
        }

        Self {
            _path:       ENV.vault_path(),
            subscribers: vec![],
            ecs,
        }
    }

    pub fn rebuild(&mut self) {
        let subs = mem::take(&mut self.subscribers);

        *self = Index::build();

        self.subscribers = subs;
    }

    pub fn delete_empty_unnamed_files(&mut self) {

        debug!("Deleting empty unnamed files");

        self.ecs.delete_empty_unnamed_files();
    }

    pub fn delete_file(&mut self, id: FileId) {

        let file = self.ecs.get(id).expect("file did not exist");

        warn!("Deleting file: {}", file.file.name);

        let file = self.ecs.remove_file(id);

        file_shit::delete_from_disk(&file.path);
    }

    fn iter_files_with_cmd<P>(&self, mut predicate: P) -> Vec<FileView>
    where
        P: FnMut(&EcsFileView) -> bool,
    {
        self
            .ecs
            .get_all   ()
            .filter    (|f| predicate(f))
            .map       (FileView::from)
            .collect()
    }

    pub fn handle_command(&mut self, command: VaultCommand) {

        macro_rules! send {
            ($resp:ident => $expr:expr) => {
                _ = $resp.send($expr)
            };
        }

        match command {
            VaultCommand::IterFilesWith  (opts, resp) => send!(resp => self.iter_files_with_cmd    (opts.filter)),
            VaultCommand::OpenInObsidian (opts, resp) => send!(resp => self.handle_open_in_obsidian(opts)),
            VaultCommand::Register       (_,    resp) => send!(resp => self.handle_register        ()),
            VaultCommand::ModifyFile     (opts, resp) => send!(resp => self.handle_modify_file     (opts)),
            VaultCommand::DeleteFile     (opts, resp) => send!(resp => self.delete_file            (opts.id)),
            VaultCommand::NukeActionables(_,    resp) => send!(resp => self.nuke_action_property   ()),
            VaultCommand::GetVaultStats  (_,    resp) => send!(resp => self.calc_vault_stats_ecs   ()),
        }
    }

    fn handle_open_in_obsidian(&self, opts: OpenInObsidian) {
        let file = self.ecs.get(opts.id).unwrap();

        obsidian::open_in_obsidian(&file.file.name);
    }

    fn handle_register(&mut self) -> Receiver<VaultUpdate> {
        let (tx, rx) = channel(1000);

        self.subscribers.push(tx);

        rx
    }

    fn handle_modify_file(&mut self, opts: ModifyFile) -> Result<(), String> {

        self.validate_modify_commands(&opts.changes)?;

        todo!();
        // let file = self.md_files.get_mut(&opts.id).unwrap();

        // opts
        //     .changes
        //     .iter()
        //     .filter_map(|command| Some(match command {
        //         ModifyFileKind::SetTypeInfo  => (FmProperty::Type, FmType::Info  .get_key()),

        //         ModifyFileKind::SetAction(_) => (FmProperty::Type, FmType::Action.get_key()),
        //         _ => None?
        //     }))
        //     .for_each(|prop| file.set_property(prop.0, prop.1))
        // ;

        // opts
        //     .changes
        //     .iter()
        //     .filter_map(|command| Some(match command {
        //         ModifyFileKind::SetAction(action) => (FmProperty::Action, action.get_key()),
        //         ModifyFileKind::SetStatus(status) => (FmProperty::Status, status.get_key()),
        //         _ => None?
        //     }))
        //     .for_each(|prop| file.set_property(prop.0, prop.1))
        // ;

        // file.write_file();

        Ok(())
    }

    fn validate_modify_commands(&self, changes: &[ModifyFileKind]) -> Result<(), String> {

        let mut info       = vec![];
        let mut action     = vec![];
        let mut status     = vec![];

        for cmd in changes.iter() {
            match cmd {
                ModifyFileKind::SetTypeInfo  => info  .push(cmd),
                ModifyFileKind::SetAction(_) => action.push(cmd),
                ModifyFileKind::SetStatus(_) => status.push(cmd),
            }
        }

        macro_rules! check {
            ($first:expr, $second:expr, $err:expr) => {
                if !$first.is_empty() && !$second.is_empty() {
                    Err($err)?
                }
            };
            ($list:expr, $err:expr) => {
                if $list.len() > 1 {
                    Err($err)?
                }
            };
        }

        check!(info, action, format!("Incompatible commands, Set Info and {:?}", action));

        check!(action, format!("Only 1 Set Action command allowed: {:?}", action));
        check!(status, format!("Only 1 Set Status command allowed: {:?}", status));

        Ok(())
    }

    fn nuke_action_property(&mut self) {
        debug!("Nuking action property");

        todo!()
        // self
        //     .iter_files_mut()
        //     .filter        (|f| f.is_type_action())
        //     .for_each      (|f| {
        //         f.remove_property(FmProperty::Action);
        //         f.remove_property(FmProperty::Status);

        //         f.write_file();
        //     })
        // ;
    }

    pub fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    pub fn ecs_mut(&mut self) -> &mut Ecs {
        &mut self.ecs
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

    let mut index = Index::build();

    // Do this before setting up the file watch
    index.delete_empty_unnamed_files();


    let (tx, rx) = mpsc::channel::<VaultCommand>(1000);

    let watcher = watch::Watcher::new(ENV.vault_path()).unwrap();

    tokio::spawn(async move {
        handle_serve(index, rx, watcher).await;
    });

    tx
}

async fn handle_serve(mut index: Index, mut rx: mpsc::Receiver<VaultCommand>, mut watcher: watch::Watcher) {

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


#[allow(dead_code)] // reason: prod select via static const
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

    // TODO: put this in a config
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


#[derive(Debug, Default)]
pub struct VaultStats {
    pub info_total:             usize,

    pub info_archived:          usize,
    pub info_complete:          usize,


    pub actionables_total:      usize,

    /// Actionables that aren't complete or archived
    pub actionables_open:       usize,

    pub actionables_complete:   usize,
    pub actionables_archived:   usize,

    pub needs_action:           usize,
    pub needs_sorted:           usize,

    pub open_todo:              usize,
    pub open_backlog:           usize,
    pub open_entertainment:     usize,
    pub open_maybe_someday:     usize,
    pub open_waiting_for:       usize,
}

impl Index {

    pub fn calc_vault_stats_ecs(&self) -> VaultStats {

        let info   = ComponentKind::Info;
        let action = ComponentKind::Action;

        VaultStats {
            info_total:           self.ecs.get_component_counts(info),
            info_archived:        self.ecs.get_all().filter(|x| x.info && x.status_eq(FmStatus::Archived ))            .count(),
            info_complete:        self.ecs.get_all().filter(|x| x.info && x.status_eq(FmStatus::Completed))            .count(),
            actionables_total:    self.ecs.get_component_counts(action),
            actionables_open:     self.ecs.get_all().filter(|x| x.is_actionable() && x.is_open())                      .count(),
            actionables_complete: self.ecs.get_all().filter(|x| x.is_actionable() && x.status_eq(FmStatus::Completed)) .count(),
            actionables_archived: self.ecs.get_all().filter(|x| x.is_actionable() && x.status_eq(FmStatus::Archived))  .count(),


            needs_action:         self.ecs.get_all().filter(|x| x.needs_action_assigned())                             .count(),
            needs_sorted:         self.ecs.get_all().filter(|x| x.needs_sorting())                                     .count(),

            open_todo:            self.ecs.get_all().filter(|x| x.is_open() && x.action_eq(FmAction::Todo))            .count(),
            open_backlog:         self.ecs.get_all().filter(|x| x.is_open() && x.action_eq(FmAction::Backlog))         .count(),
            open_entertainment:   self.ecs.get_all().filter(|x| x.is_open() && x.action_eq(FmAction::Entertainment))   .count(),
            open_maybe_someday:   self.ecs.get_all().filter(|x| x.is_open() && x.action_eq(FmAction::MaybeSomeday))    .count(),
            open_waiting_for:     self.ecs.get_all().filter(|x| x.is_open() && x.action_eq(FmAction::WaitingFor))      .count(),
        }

    }
}
