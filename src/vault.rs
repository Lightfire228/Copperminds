use std::{env, path::PathBuf};

use walkdir::{DirEntry, WalkDir};


pub struct Index {
    pub md_files: Vec<DirEntry>
}

impl Index {
    pub fn build() -> Self {
        let files = scan_vault();

        let md_files = md_files(&files);



        Self {
            md_files,
        }
    }
}


fn vault_folder() -> PathBuf {
    let home = env::home_dir().unwrap();

    home.join("Notes")
}

fn scan_vault() -> Vec<DirEntry> {
    WalkDir::new(vault_folder())
        .into_iter   ()
        .filter_entry(|e| !is_hidden(e))
        .filter_map  (|e| e.ok())
        .collect     ()
}

fn md_files(files: &[DirEntry]) -> Vec<DirEntry> {
    files
        .iter  ()
        .filter(|f| ends_with(f, ".md"))
        .map   (|f| f.clone())
        .collect()
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

