
pub mod fm;
pub mod command;
pub mod ecs;

mod watch;
mod generator;


use crate::{config::Config, file_shit, obsidian, vault::{command::{ModifyFile, ModifyFileKind, OpenInObsidian, VaultCommand, VaultUpdate}, ecs::{Ecs, NewFile, components::*}, fm::*}};
use file_id::FileId;
use futures::future::join_all;
use log::{debug};
use std::{env, mem, path::{Path, PathBuf}, sync::LazyLock, usize};
use crate::prelude::*;

use tokio::{select, sync::{mpsc::{self, Sender, Receiver, channel}}};
use walkdir::{DirEntry, WalkDir};

// pub const ENV: Env = Env::Prod;

macro_rules! build_regex {
    (crate $reg_crate:ident, $i:ident = $r:expr) => {

        use $reg_crate::Regex;
        use std::sync::LazyLock;

        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());

    };
    ($i:ident = $r:expr) => {
        build_regex!(crate regex, $i = $r)
    };

    (fancy $i:ident = $r:expr) => {
        build_regex!(crate fancy_regex, $i = $r)
    };
}

pub(crate) use build_regex;


pub struct Index {
    subscribers: Vec<Sender<VaultUpdate>>,
    ecs:         Ecs,
    config:      Config,
}

#[derive(Debug, Clone, Eq)]
pub struct FileView {
    pub id:   FileId,
    pub name: String,
}


impl Index {
    pub fn build(config: Config) -> Self {
        let mut ecs = Ecs::new();

        let files = scan_vault(&config)
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
            subscribers: vec![],
            ecs,
            config,
        }
    }

    pub fn rebuild(&mut self) {
        let subs = mem::take(&mut self.subscribers);
        let conf = mem::take(&mut self.config);

        *self = Index::build(conf);

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

        obsidian::open_in_obsidian(&self.config, &file.file.name);
    }

    fn handle_register(&mut self) -> Receiver<VaultUpdate> {
        let (tx, rx) = channel(1000);

        self.subscribers.push(tx);

        rx
    }

    fn handle_modify_file(&mut self, opts: ModifyFile) -> Result<(), String> {

        self.validate_modify_commands(&opts.changes)?;

        for change in &opts.changes {
            match change {
                ModifyFileKind::SetTypeInfo  => self.ecs.set_info  (opts.id),
                ModifyFileKind::SetAction(a) => self.ecs.set_action(opts.id, *a),
                ModifyFileKind::SetStatus(s) => self.ecs.set_status(opts.id, *s),
            }
        }


        self.ecs.write_to_disk(opts.id);

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

fn scan_vault(config: &Config) -> impl Iterator<Item = DirEntry> {
    WalkDir::new(&config.vault_path)
        .into_iter   ()
        .filter_entry(|e| !is_hidden(e) && !is_excluded(e.path(), config))
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


fn is_excluded(path: &Path, config: &Config) -> bool {
    path
        .ancestors()
        .any      (|path| config
            .folder_excludes
            .iter()
            .any(|exclude| {
                path.ends_with(exclude)
            })
        )
}



pub fn serve(config: &Config) -> Sender<VaultCommand> {

    let mut index = Index::build(config.clone());

    // Do this before setting up the file watch
    index.delete_empty_unnamed_files();


    let (tx, rx) = mpsc::channel::<VaultCommand>(1000);

    let watcher = watch::Watcher::new(config.vault_path.clone()).unwrap();

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


#[allow(dead_code)] // reason: prod select via const in main
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Prod,
    Dev,
}

impl Env {

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

    pub project_files:          usize,
    // pub unqiue_projects:        usize,
}

impl Index {

    pub fn calc_vault_stats_ecs(&self) -> VaultStats {



        VaultStats {
            info_total:           self.ecs.get_component_counts::<InfoComponent>(),
            info_archived:        self.ecs.get_all().filter(|x| x.is_info() && x.status_eq(FmStatus::Archived ))       .count(),
            info_complete:        self.ecs.get_all().filter(|x| x.is_info() && x.status_eq(FmStatus::Completed))       .count(),
            actionables_total:    self.ecs.get_component_counts::<ActionComponent>(),
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

            project_files:        self.ecs.get_component_counts::<ProjectComponent>(),
            // unqiue_projects:      self.ecs.iter_components::<ProjectComponent>().map(|x| )
        }

    }
}



impl PartialEq for FileView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


impl<'a> From<EcsFileView<'a>> for FileView {
    fn from(value: EcsFileView) -> Self {
        FileView {
            id:   value.id,
            name: value.file.name.clone(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_excluded() {
        let config = Config::with_excludes(vec![
            "exclude"     .to_string(),
            "also exclude".to_string(),
        ]);

        let test = |name| is_excluded(&PathBuf::from(name), &config);


        assert_eq!(test("should/exclude/file.md"),      true);
        assert_eq!(test("should/exclude"),              true);
        assert_eq!(test("should/also exclude/file.md"), true);
        assert_eq!(test("should/not exclude/file.md"),  false);
        assert_eq!(test("should/not/exclude.md"),       false);

    }
}
