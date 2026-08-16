
pub mod md_file;
pub mod fm;
pub mod command;

mod file_utilities;
mod watch;


use crate::{obsidian, backup, vault::{command::VaultCommand, md_file::{FileId, FileView}}};
use std::{collections::HashMap, env, path::PathBuf};

use tokio::{select, sync::mpsc::{self, Sender}};
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
    md_files: HashMap<FileId, MdFile>,
    _path:    PathBuf,
    next_id:  usize,
}

impl Index {
    pub fn build() -> Self {
        let files    = scan_vault();

        let mut next = 0;

        let md_files = files
            .filter   (|f| ends_with(f, ".md"))
            .map      (|f| {
                let path = f.path().to_path_buf();
                let id   = next;
                next += 1;

                (id, MdFile::new(id, path))
            })
            .collect()
        ;

        Self {
            md_files,
            _path:   vault_folder(),
            next_id: next,
        }
    }

    #[allow(unused)]
    pub fn backup(&self) {
        println!("Backing up vault");

        backup::backup(&self._path);
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

        match command {
            VaultCommand::IterFilesWith(filter, resp) => {
                resp.send(self
                    .iter_files_with_cmd(filter.filter)
                )
                .unwrap()
            },

            VaultCommand::SetProperty(prop, resp) => {
                resp.send(self
                    .get_file_mut(prop.id)
                    .set_property(prop.prop, prop.value)
                )
                .unwrap()
            },

            VaultCommand::OpenInObsidian(opts, resp) => {
                let file = &self.md_files[&opts.id];

                obsidian::open_in_obsidian(file);

                resp.send(()).unwrap()
            }
        }
    }
}


pub fn vault_folder() -> PathBuf {
    let home = env::home_dir().unwrap();

    home.join(ENV.vault())
}

fn scan_vault() -> impl Iterator<Item = DirEntry> {
    WalkDir::new(vault_folder())
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

    let watcher = watch::Watcher::new(vault_folder()).unwrap();

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
                index.handle_watch_event(event).await;
            }
        };

    }
}

impl Index {
    async fn handle_watch_event(&mut self, event: Option<watch::ModificationType>) {
        let Some(event) = event else {
            return;
        };

        // TODO: when i hit `ctrl + s` in obsidian, I get 2 update events
        //       do i need to "debounce" events by a few millis?

        type Mt = watch::ModificationType;
        match event {
            Mt::Create { target }   => self.handle_create_event(target),
            Mt::Update { target }   => self.handle_update_event(target),
            Mt::Delete { target }   => self.handle_delete_event(target),
            Mt::Rename { from, to } => self.handle_rename_event(from, to),
        }
    }

    fn handle_create_event(&mut self, target: PathBuf) {
        if !target.ends_with(".md") {
            return;
        }

        println!("Adding file to index: {:?}", &target);


        let id = self.next_id;
        self.next_id += 1;

        let md_file = MdFile::new(id, target);

        self.md_files.insert(id, md_file);
    }

    fn handle_update_event(&mut self, target: PathBuf) {
        if !target.ends_with(".md") {
            return;
        }

        println!("Updating file: {:?}", &target);

        todo!()
    }

    fn handle_delete_event(&mut self, target: PathBuf) {
        if !target.ends_with(".md") {
            return;
        }

        println!("Deleting file: {:?}", &target);

        todo!()
    }

    fn handle_rename_event(&mut self, from: PathBuf, to: PathBuf) {

        match (from.ends_with(".md"), to.ends_with(".md")) {
            (true, false)  => return self.handle_delete_event(from),
            (false, true)  => return self.handle_create_event(to),
            (false, false) => return,
            _              => {}
        }

        println!("Rename");
        println!("  - from: {:?}", from);
        println!("  - to:   {:?}", to);

        todo!()
    }

}


pub enum Env {
    Prod,
    Dev,
}

impl Env {
    pub fn vault(&self) -> &'static str {
        match self {
            Self::Prod => "Notes",
            Self::Dev  => "Notes_dev",
        }
    }
}
