use std::{collections::HashMap, env, fs, mem, ops::Deref, path::PathBuf, sync::LazyLock};

use regex::Regex;
use serde::de;
use walkdir::{DirEntry, WalkDir};
use yaml_serde::{Mapping, Value};
use trash;

macro_rules! regex {
    ($i:ident = $r:expr) => {
        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
    };
}

pub(crate) use regex;


pub struct Index {
    pub md_files: Vec<Box<MdFile>>,
    
}

pub struct MdFile {
    pub entry:       DirEntry,
    pub frontmatter: Option<Mapping>,
    pub inbox:       Option<String>,
    pub file_name:   String,
    pub md_text:     String,
    pub raw_text:    String,
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
            .map      (|f| parse_md_file(&f))
            .collect  ()
        ;

        Self {
            md_files,
        }
    }

    pub fn needs_inbox(&self) -> impl Iterator<Item = &MdFile> {
        self.md_files
            .iter   ()
            .filter (|f| f.inbox.is_none())
            .map    (|f| f.as_ref())
    }
    
    pub fn needs_inbox_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {
        self.md_files
            .iter_mut()
            .filter  (|f| f.inbox.is_none())
            .map     (|f| f.as_mut())
    }
    
    pub fn all_files_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {
        self.md_files
            .iter_mut()
            .map     (|f| f.as_mut())
    }

    pub fn bulk_assign_inbox<F>(&mut self, inbox: &str, target: BulkAssign, filter: F)
    where 
        F: Fn(&MdFile) -> bool
    {
        let files: Box<dyn Iterator<Item = &mut MdFile>> = match target {
            BulkAssign::All            => Box::new(self.all_files_mut  ()),
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

    pub fn list_all_inboxes(&self) -> HashMap<&str, Vec<&MdFile>> {
        let files = self.md_files
            .iter      ()
            .filter_map(|f| f.inbox

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
        regex!(RE = r"^\s*$");

        self.list_unnamed_files()
            .filter(|f| RE.is_match(&f.md_text)) // TODO: use raw_text instead?
    }

    pub fn list_unnamed_files(&self) -> impl Iterator<Item = &MdFile> {

        regex!(RE = r"^([\d \-_]*|Untitled.*?)\.md$");

        self.md_files
            .iter  ()
            .filter(|f|
                RE.is_match(&f.file_name)
            )
            .map(|f| f.as_ref())
    }

    pub fn delete_empty_files(&mut self) {
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


impl MdFile {

    pub fn assign_inbox(&mut self, inbox: String) {
        
        let mut fm = self.frontmatter.take().unwrap_or_else(|| Mapping::new());
        
        let key = Value::String("inbox".to_owned());
        let val = Value::String(inbox.clone());
        fm.insert(key, val);

        self.inbox       = Some(inbox);
        self.frontmatter = Some(fm);
    }

    pub fn write_file(&self) {
        let Some(fm) = &self.frontmatter else {
            fs::write(self.entry.path(), &self.md_text).unwrap();
            return;
        };

        let fm_text = yaml_serde::to_string(fm).unwrap();
        let text    = format!("---\n{fm_text}---\n{}", self.md_text);

        fs::write(self.entry.path(), &text).unwrap();
    }
}

impl PartialEq for MdFile {
    fn eq(&self, other: &Self) -> bool {
        self.entry.path() == other.entry.path()
    }
}

impl Eq for MdFile {}


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


fn parse_md_file(file: &DirEntry) -> Box<MdFile> {

    let name = || file.file_name().to_str().unwrap().to_owned();
    
    
    let blank = |text: String| {
        Box::new(MdFile {
            entry:       file.clone(),
            frontmatter: None,
            inbox:       None,
            file_name:   name(),
            md_text:     text.clone(),
            raw_text:    text,
        })
    };

    let parsed = get_frontmatter_text(&file);

    let parsed = match parsed {
        Parsed::None       (text)      => return blank(text),
        Parsed::Frontmatter(parsed_fm) => parsed_fm
    };

    let Some(fm)   = yaml_serde::from_str::<Mapping>(&parsed.fm).ok() else {
        return blank(parsed.raw_text);
    };

    Box::new(MdFile {
        entry:       file.clone(),
        inbox:       fm
            .get     ("inbox")
            .and_then(|i| i.as_str())
            .map     (|i| i.to_owned())
        ,
        frontmatter: Some(fm),
        file_name:   name(),
        md_text:     parsed.body,
        raw_text:    parsed.raw_text,
    })

}


fn get_frontmatter_text(file: &DirEntry) -> Parsed {

    // (?ms) set flags
    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    regex!(RE = r"^(?ms)---\s*(.+?)^---\s*$(.*)");

    let text = fs::read_to_string(file.path()).unwrap();

    let Some(captures) = RE.captures(&text) else {
        return Parsed::None(text);
    };

    let fm   = captures[1].to_owned();
    let body = captures[2].to_owned();

    Parsed::Frontmatter(ParsedFm {
        raw_text: text,
        fm,
        body,
    })

}

enum Parsed {
    None       (String),
    Frontmatter(ParsedFm),
}

struct ParsedFm {
    pub raw_text: String,
    pub fm:       String,
    pub body:     String,
}