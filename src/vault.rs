
pub mod md_file;
pub mod fm;
pub mod command;

mod file_utilities;
mod watch;
mod generator;


use crate::{obsidian, vault::{command::{ModifyFile, ModifyFileKind, OpenInObsidian, VaultCommand, VaultUpdate}, fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}, md_file::FileView, watch::FileData}};
use file_id::FileId;
use futures::future::join_all;
use log::{debug};
use std::{collections::HashMap, env, mem, path::{Path, PathBuf}, usize};
use crate::prelude::*;

use tokio::{select, sync::{mpsc::{self, Sender, Receiver, channel}}};
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
// TODO: restructure this like an ECS
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

        debug!("Deleting empty unnamed files");

        let files: Vec<_> = self
            .get_empty_unnamed_files()
            .cloned ()
            .collect()
        ;

        for id in files.iter() {
            let path = &self.md_files[id].path;

            trash::delete(path).unwrap();
            self.md_files.remove(id);
        }
    }

    pub fn delete_file(&mut self, id: FileId) {

        let file = self.get_file(id);

        warn!("Deleting file: {}", file.file_name);

        let path = &self.md_files[&id].path;

        trash::delete(path).unwrap();
        self.md_files.remove(&id);
    }

    fn get_empty_unnamed_files(&self) -> impl Iterator<Item = &FileId> {
        self
            .iter_files()
            .filter(|f| f.is_empty_raw() && f.is_unnamed())
            .map   (|f| &f.id)

    }

    fn iter_files(&self) -> impl Iterator<Item = &MdFile> {
        self
            .md_files
            .iter()
            .map (|f| f.1)
    }

    fn iter_files_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {
        self
            .md_files
            .iter_mut()
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
            VaultCommand::IterFilesWith  (opts, resp) => send!(resp => self.iter_files_with_cmd    (opts.filter)),
            VaultCommand::OpenInObsidian (opts, resp) => send!(resp => self.handle_open_in_obsidian(opts)),
            VaultCommand::Register       (_,    resp) => send!(resp => self.handle_register        ()),
            VaultCommand::ModifyFile     (opts, resp) => send!(resp => self.handle_modify_file     (opts)),
            VaultCommand::DeleteFile     (opts, resp) => send!(resp => self.delete_file            (opts.id)),
            VaultCommand::GetVaultStats  (_,    resp) => send!(resp => self.calc_vault_stats       ()),
            VaultCommand::NukeActionables(_,    resp) => send!(resp => self.nuke_action_property   ()),
        }
    }

    fn handle_open_in_obsidian(&self, opts: OpenInObsidian) {
        let file = &self.md_files[&opts.id];

        obsidian::open_in_obsidian(file);
    }

    fn handle_register(&mut self) -> Receiver<VaultUpdate> {
        let (tx, rx) = channel(1000);

        self.subscribers.push(tx);

        rx
    }

    fn handle_modify_file(&mut self, opts: ModifyFile) -> Result<(), String> {

        self.validate_modify_commands(&opts.changes)?;

        let file = self.get_file_mut(opts.id);

        opts
            .changes
            .iter()
            .filter_map(|command| Some(match command {
                ModifyFileKind::SetTypeInfo           => (FmProperty::Type, FmType::Info  .get_key()),

                ModifyFileKind::SetActionTodo         |
                ModifyFileKind::SetActionWaitingFor   |
                ModifyFileKind::SetActionProject      |
                ModifyFileKind::SetActionMaybeSomeday => (FmProperty::Type, FmType::Action.get_key()),
                _ => None?
            }))
            .for_each(|prop| file.set_property(prop.0, prop.1))
        ;

        opts
            .changes
            .iter()
            .filter_map(|command| Some(match command {
                ModifyFileKind::SetActionTodo         => (FmProperty::Action, FmAction::Todo        .get_key()),
                ModifyFileKind::SetActionWaitingFor   => (FmProperty::Action, FmAction::WaitingFor  .get_key()),
                ModifyFileKind::SetActionProject      => (FmProperty::Action, FmAction::Project     .get_key()),
                ModifyFileKind::SetActionMaybeSomeday => (FmProperty::Action, FmAction::MaybeSomeday.get_key()),
                ModifyFileKind::SetStatusComplete     => (FmProperty::Status, FmStatus::Completed   .get_key()),
                ModifyFileKind::SetStatusArchived     => (FmProperty::Status, FmStatus::Archived    .get_key()),
                _ => None?
            }))
            .for_each(|prop| file.set_property(prop.0, prop.1))
        ;

        file.write_file();

        Ok(())
    }

    fn validate_modify_commands(&self, changes: &[ModifyFileKind]) -> Result<(), String> {

        let mut info       = vec![];
        let mut action     = vec![];
        let mut status     = vec![];

        for cmd in changes.iter() {
            match cmd {
                ModifyFileKind::SetTypeInfo           => info.push(cmd),

                ModifyFileKind::SetActionTodo         |
                ModifyFileKind::SetActionWaitingFor   |
                ModifyFileKind::SetActionProject      |
                ModifyFileKind::SetActionMaybeSomeday => action.push(cmd),

                ModifyFileKind::SetStatusComplete     |
                ModifyFileKind::SetStatusArchived     => status.push(cmd),
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

        self
            .iter_files_mut()
            .filter        (|f| f.is_type_action())
            .for_each      (|f| {
                f.remove_property(FmProperty::Action);
                f.remove_property(FmProperty::Status);

                f.write_file();
            })
        ;




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
}

impl Index {
    pub fn calc_vault_stats(&self) -> VaultStats {

        VaultStats {
            info_total:           self.count(|x| x.is_type_info  ()                   ),
            info_archived:        self.count(|x| x.is_type_info  () && x.is_archived()),
            info_complete:        self.count(|x| x.is_type_info  () && x.is_complete()),
            actionables_total:    self.count(|x| x.is_type_action()                   ),
            actionables_open:     self.count(|x| x.is_type_action() && x.is_open()    ),
            actionables_complete: self.count(|x| x.is_type_action() && x.is_complete()),
            actionables_archived: self.count(|x| x.is_type_action() && x.is_archived()),

            needs_action:         self.count(|x| x.needs_action_assigned()),
            needs_sorted:         self.count(|x| x.needs_sorting()),
        }

    }

    fn count<T>(&self, x: T) -> usize
    where
        T: FnMut(&&MdFile) -> bool
    {
        self
            .iter_files()
            .filter    (x)
            .count     ()
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    fn id(i: usize) -> FileId {
        FileId::Inode {
            device_id:    0,
            inode_number: i as u64,
        }
    }

    fn build_empty_unnamed_test_cases(
        empty_titles:     &[String],
        non_empty_titles: &[String],
        non_empty_bodies: &[&str],
    )
        -> Vec<Test>
    {

        let mut i = 0;

        macro_rules! test {
            ($n:expr, $b:expr, $e:expr) => {{
                i += 1;
                Test {
                    id:            id(i),
                    file_name:     $n.to_owned(),
                    file_body:     $b.to_owned(),
                    empty_unnamed: $e,
                }
            }};
        }

        let mut test_cases = vec![];

        for empty_title in empty_titles.iter() {
            test_cases.push(test!(empty_title, "",      true));
            test_cases.push(test!(empty_title, " \t\n", true));

            for body in non_empty_bodies.iter() {
                test_cases.push(test!(empty_title, *body, false));
            }
        }

        for non_empty_title in non_empty_titles.iter() {
            test_cases.push(test!(non_empty_title, "",      false));
            test_cases.push(test!(non_empty_title, " \t\n", false));

            for body in non_empty_bodies.iter() {
                test_cases.push(test!(non_empty_title, *body, false));
            }
        }

        test_cases
    }




    #[derive(Debug)]
    struct Test {
        id:            FileId,
        file_name:     String,
        file_body:     String,
        empty_unnamed: bool,
    }

    #[test]
    fn test_empty_unnamed_files() {
        let empty_titles = [
            "Untitled",
            "untitled",
            "Untitled - 1",
            "Untitled (2)",
            "2026-01-01",
            "2026-01-01 ",
            "2026-01-01 - 00_00_00",
            "2026-01-01 - 00",
            "2026",
            "1",
            "___ ---",
        ]
            .map(|f| format!("{f}.md"))
        ;

        let non_empty_titles = [
            "Untitledtropolis",
            "Untitled-thingy",
            "2026-01-01 - 00_00_00 - titled",
            "2026-01-01 - titled",
            "2026-01-01 titled",
            "2026-01 there are rats in my basement",
            "dorktastic",
        ]
            .map(|f| format!("{f}.md"))
        ;

        let non_empty_bodies = [
            "---\n\n---\n",
            ".",
        ];

        let test_cases = build_empty_unnamed_test_cases(&empty_titles, &non_empty_titles, &non_empty_bodies);

        let files: HashMap<FileId, MdFile> = test_cases
            .iter()
            .map      (|t| (t.id.clone(), MdFile::test_parse(t.id.clone(), t.file_name.clone(), t.file_body.clone())))
            .collect  ()
        ;

        let vault = Index {
            md_files:    files,
            subscribers: vec![],
            path:        PathBuf::new(),
        };

        let empty_unamed: Vec<_> = vault.get_empty_unnamed_files().collect();

        println!("{empty_unamed:?}");

        for file in test_cases {

            println!("{file:?}");

            assert_eq!(empty_unamed.contains(&&file.id), file.empty_unnamed)
        }

    }

}
