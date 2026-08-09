
pub mod md_file;
pub mod fm;
pub mod command;

mod file_utilities;


use crate::{backup, vault::{command::VaultCommand, md_file::{FileId, FileView}}};
use std::{collections::HashMap, env, path::PathBuf};

use tokio::sync::mpsc::{self, Sender};
use walkdir::{DirEntry, WalkDir};
use trash;

use md_file::{MdFile};


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
            _path: vault_folder(),
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
        }
    }
}


pub fn vault_folder() -> PathBuf {
    let home = env::home_dir().unwrap();

    home.join("Notes")
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

    let (tx, mut rx) = mpsc::channel::<VaultCommand>(1000);

    tokio::spawn(async move {
        let mut index = Index::build();

        index.delete_empty_unnamed_files();

        while let Some(command) = rx.recv().await {
            index.handle_command(command);
        }
    });

    tx
}
