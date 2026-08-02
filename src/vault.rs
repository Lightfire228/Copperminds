
pub mod md_file;

mod file_utilities;


use crate::{backup, vault::md_file::FileId};
use std::{collections::HashMap, env, ops::Deref, path::PathBuf};

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
    pub md_files: HashMap<FileId, Box<MdFile>>,
    pub _path:    PathBuf,
}

impl Index {
    pub fn build() -> Self {
        let files    = scan_vault();

        let mut next = 0;

        let md_files = files
            .filter   (|f| ends_with(f, ".md"))
            .map      (|f| {
                let path = f.path().to_path_buf();
                let id = next;
                next += 1;

                (id, Box::new(MdFile::new(id, path)))
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
            .map (|f| f.1.deref())
    }

    pub fn iter_files_with<P>(&self, mut predicate: P) -> impl Iterator<Item = FileId>
    where
        P: FnMut(&MdFile) -> bool,
    {
        self
            .md_files
            .iter()
            .filter(move |f| predicate(f.1.deref()))
            .map (|f| *f.0)
    }

    pub fn _get_file(&self, id: FileId) -> &MdFile {
        &self.md_files[&id]
    }

    pub fn get_file_mut(&mut self, id: FileId) -> &mut MdFile {
        self.md_files.get_mut(&id).unwrap()
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


#[cfg(test)]
mod tests {
    use std::{path::Path};

    use crate::vault::md_file::FmProperty;

    use super::*;


    fn load_file(name: &str) -> MdFile {
        let dir  = format!("{}/test_files/{name}", env!("CARGO_MANIFEST_DIR"));
        let path = Path::new(&dir).to_path_buf();

        MdFile::new(0, path)
    }


    #[test]
    fn test_type_sorting() {
        let untyped = load_file("sorting/type_none.md");
        let info    = load_file("sorting/type_info.md");
        let action  = load_file("sorting/type_action.md");

        assert_eq!(untyped.is_untyped(), true);
        assert_eq!(info   .is_untyped(), false);
        assert_eq!(action .is_untyped(), false);

        assert_eq!(untyped.is_actionable(), false);
        assert_eq!(info   .is_actionable(), false);
        assert_eq!(action .is_actionable(), true);

        assert!(info  .is_property(FmProperty::Type, "info"));
        assert!(action.is_property(FmProperty::Type, "action"));
    }

    #[test]
    fn test_status_sorting() {
        let archive   = load_file("sorting/status_archive.md");
        let archived  = load_file("sorting/status_archived.md");
        let complete  = load_file("sorting/status_complete.md");
        let completed = load_file("sorting/status_completed.md");

        assert_eq!(archive  .is_archived(), true);
        assert_eq!(archived .is_archived(), true);
        assert_eq!(complete .is_archived(), false);
        assert_eq!(completed.is_archived(), false);

        assert_eq!(archive  .is_complete(), false);
        assert_eq!(archived .is_complete(), false);
        assert_eq!(complete .is_complete(), true);
        assert_eq!(completed.is_complete(), true);
    }

}
