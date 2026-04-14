pub mod md_file;

use std::{collections::HashMap, env, fs, mem, ops::Deref, path::PathBuf};

use serde::de;
use walkdir::{DirEntry, WalkDir};
use yaml_serde::{Mapping, Sequence, Value};
use trash;

use md_file::{MdFile, Frontmatter, FmProperty, FmPropertyList};

macro_rules! regex {
    ($i:ident = $r:expr) => {
        use regex::Regex;
        use std::sync::LazyLock;

        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
    };
}

pub(crate) use regex;

use crate::{backup};


pub struct Index {
    pub md_files: Vec<Box<MdFile>>,
    pub path:     PathBuf,
    
}

pub enum BulkAssign {
    All,
    NeedsInboxOnly,
}

impl Index {
    pub fn build() -> Self {
        let files    = scan_vault();

        let md_files = files
            .filter   (|f| ends_with(f, ".md"))
            .map      (|f| MdFile::new(&f))
            .map      (|f| Box   ::new(f))
            .collect  ()
        ;

        Self {
            md_files,
            path: vault_folder(),
        }
    }

    pub fn backup(&self) {
        println!("Backing up vault");

        backup::backup(&self.path);
    }

    pub fn iter_files(&self) -> impl Iterator<Item = &MdFile> {
        self.md_files
            .iter()
            .map (|f| f.as_ref())
    }

    pub fn iter_files_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {
        self.md_files
            .iter_mut()
            .map (|f| f.as_mut())
    }

    pub fn needs_inbox(&self) -> impl Iterator<Item = &MdFile> {

        self.iter_files()
            .filter    (|f| f.frontmatter.inbox.is_none())
    }
    
    pub fn needs_inbox_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {

        self.iter_files_mut()
            .filter  (|f| f.frontmatter.inbox.is_none())
    }
    
    pub fn bulk_assign_property<F>(&mut self, property: FmProperty, value: &str, filter: F)
    where 
        F: Fn(&MdFile) -> bool
    {
        let files = self.iter_files_mut()
            .filter(|f| filter(&f))
        ;

        let mut count = 0;

        for file in files {
            count += 1;

            file.assign_property(property, value.to_owned());
            file.write_file();
        }

        println!("assigned: {}", count);
    }
    
    pub fn bulk_assign_inbox<F>(&mut self, inbox: &str, target: BulkAssign, filter: F)
    where 
        F: Fn(&MdFile) -> bool
    {
        let files: Box<dyn Iterator<Item = &mut MdFile>> = match target {
            BulkAssign::All            => Box::new(self.iter_files_mut ()),
            BulkAssign::NeedsInboxOnly => Box::new(self.needs_inbox_mut()),
        };

        let filtered = files
            .filter(|f| filter(&f))
        ;

        let mut count = 0;

        for file in filtered {
            count += 1;

            file.assign_inbox(inbox.to_owned());
            file.write_file();
        }

        println!("assigned: {}", count);
    }
    
    pub fn bulk_assign_processing_tag<F>(&mut self, tag: &str, filter: F)
    where 
        F: Fn(&MdFile) -> bool
    {
        let files = self.iter_files_mut()
            .filter(|f| filter(&f))
        ;

        let mut count = 0;

        for file in files {
            count += 1;

            file.assign_processing_tag(tag.to_owned());
            file.write_file();
        }

        println!("assigned: {}", count);
    }

    pub fn list_all_inboxes(&self) -> HashMap<&str, Vec<&MdFile>> {

        let files = self
            .iter_files()
            .filter_map(|f| f.frontmatter.inbox
                .as_ref()
                .map   (|i| (i.as_str(), f))
            )
        ;

        let mut map: HashMap<&str, Vec<&MdFile>> = HashMap::new();

        for (inbox, file) in files {
            map
                .entry     (inbox)
                .and_modify(|list| list.push(file))
                .or_insert (vec![file])
            ;
        }
        
        map
    }


    pub fn list_empty_unnamed_files(&self) -> impl Iterator<Item = &MdFile> {

        self.list_unnamed_files()
            .filter(|f| f.is_empty())
    }

    pub fn list_unnamed_files(&self) -> impl Iterator<Item = &MdFile> {

        self.iter_files()
            .filter(|f| f.is_unnamed())
    }

    pub fn delete_empty_unnamed_files(&mut self) {
        
        let files: Vec<_> = self.list_empty_unnamed_files().collect();

        for file in files.iter() {
            trash::delete(file.entry.path()).unwrap();
        }

        let deleted: Vec<_> = files
            .iter   ()
            .map    (|f| *f as *const MdFile)
            .collect()
        ;

        // TODO: maybe store the files as a hashmap?
        self.md_files
            .retain(|f| !deleted.contains(
                &(f.deref() as *const MdFile)
            ))
        ;
    }

}


fn vault_folder() -> PathBuf {
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
