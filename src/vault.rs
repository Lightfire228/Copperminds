#![allow(dead_code)]

pub mod md_file;

mod file_utilities;


use crate::{backup};
use std::{collections::HashMap, env, ops::Deref, path::PathBuf};

use walkdir::{DirEntry, WalkDir};
use trash;

use md_file::{MdFile, FmProperty, FmPropertyList};


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
    pub md_files: Vec<Box<MdFile>>,
    pub path:     PathBuf,

}

pub enum BulkAssign {
    All,
    NeedsCategoryOnly,
}

impl Index {
    pub fn build() -> Self {
        let files    = scan_vault();

        let md_files = files
            .filter   (|f| ends_with(f, ".md"))
            .map      (|f| MdFile::new(f))
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
}

fn needs_category_filter(file: &MdFile) -> bool {
    file.is_uncategorized()
}

impl Index {
    pub fn needs_category(&self) -> impl Iterator<Item = &MdFile> {
        self.iter_files()    .filter(|f| needs_category_filter(*f))
    }

    pub fn needs_category_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {
        self.iter_files_mut().filter(|f| needs_category_filter(*f))
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

            file.set_property(property, value.to_owned());
            file.write_file();
        }

        println!("assigned: {}", count);
    }

    pub fn bulk_assign_category<F>(&mut self, category: &str, target: BulkAssign, filter: F)
    where
        F: Fn(&MdFile) -> bool
    {
        let files: Box<dyn Iterator<Item = &mut MdFile>> = match target {
            BulkAssign::All               => Box::new(self.iter_files_mut    ()),
            BulkAssign::NeedsCategoryOnly => Box::new(self.needs_category_mut()),
        };

        let filtered = files
            .filter(|f| filter(&f))
        ;

        let mut count = 0;

        for file in filtered {
            count += 1;

            file.set_property(FmProperty::Category, category.to_owned());
            file.write_file();
        }

        println!("assigned: {}", count);
    }

    // pub fn bulk_assign_processing_tag<F>(&mut self, tag: &str, filter: F)
    // where
    //     F: Fn(&MdFile) -> bool
    // {
    //     let files = self.iter_files_mut()
    //         .filter(|f| filter(&f))
    //     ;

    //     let mut count = 0;

    //     for file in files {
    //         count += 1;

    //         file.push_list_val(FmPropertyList::Processing, tag.to_owned());
    //         file.write_file();
    //     }

    //     println!("assigned: {}", count);
    // }

    pub fn list_all_categories(&self) -> HashMap<String, Vec<&MdFile>> {

        let files = self
            .iter_files()
            .filter_map(|f| {
                Some((
                    f.get_property(FmProperty::Category)?,
                    f
                ))

            })
        ;

        let mut map: HashMap<String, Vec<&MdFile>> = HashMap::new();

        for (category, file) in files {
            map
                .entry     (category)
                .and_modify(|list| list.push(file))
                .or_insert (vec![file])
            ;
        }

        map
    }


    pub fn delete_empty_unnamed_files(&mut self) {

        let files: Vec<_> = self
            .iter_files()
            .filter(|f| f.is_empty() && f.is_unnamed())
            .collect()
        ;

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
            .retain(|f| {
                let ptr = f.deref() as *const MdFile;

                !deleted.contains(&ptr)
            })
        ;
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

    use super::*;


    fn load_file(name: &str) -> MdFile {
        let dir  = format!("{}/test_files/{name}", env!("CARGO_MANIFEST_DIR"));
        let path = Path::new(&dir);

        let entry = walkdir::WalkDir::new(path).into_iter().next().unwrap().unwrap();

        MdFile::new(entry)
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
