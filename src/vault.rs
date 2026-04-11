use std::{collections::{HashMap}, env, fs, path::PathBuf, sync::LazyLock};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use yaml_serde::{Mapping, Value};

macro_rules! regex {
    ($i:ident = $r:expr) => {
        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
    };
}

pub(crate) use regex;


pub struct Index {
    pub md_files: Vec<MdFile>,
    
}

pub struct MdFile {
    pub entry:       DirEntry,
    pub frontmatter: Option<Mapping>,
    pub inbox:       Option<String>,
    pub file_name:   String,
    pub text:        String,
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
    }
    
    pub fn needs_inbox_mut(&mut self) -> impl Iterator<Item = &mut MdFile> {
        self.md_files
            .iter_mut()
            .filter  (|f| f.inbox.is_none())
    }

    pub fn bulk_assign_inbox_by_name<F>(&mut self, inbox: &str, filter: F)
        where F: Fn(&MdFile) -> bool
    {

        let filtered = self.needs_inbox_mut()
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
            fs::write(self.entry.path(), &self.text).unwrap();
            return;
        };

        let fm_text = yaml_serde::to_string(fm).unwrap();
        let text    = format!("---\n{fm_text}---\n{}", self.text);

        fs::write(self.entry.path(), &text).unwrap();
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


fn parse_md_file(file: &DirEntry) -> MdFile {

    let name = || file.file_name().to_str().unwrap().to_owned();
    
    
    let blank = |text| {
        MdFile {
            entry:       file.clone(),
            frontmatter: None,
            inbox:       None,
            file_name:   name(),
            text,
        }
    };

    let (fm_text, text) = get_frontmatter_text(&file);

    let Some(fm_text) = fm_text else {
        return blank(text);
    };

    let Some(fm)   = yaml_serde::from_str::<Mapping>(&fm_text).ok() else {
        return blank(text);
    };

    MdFile {
        entry:       file.clone(),
        inbox:       fm
            .get     ("inbox")
            .and_then(|i| i.as_str())
            .map     (|i| i.to_owned())
        ,
        frontmatter: Some(fm),
        file_name:   name(),
        text,
    }

}


fn get_frontmatter_text(file: &DirEntry) -> (Option<String>, String) {

    // (?ms) set flags
    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    regex!(RE = r"^(?ms)---\s*(.+?)^---\s*$(.*)");

    let text = fs::read_to_string(file.path()).unwrap();

    let Some(captures) = RE.captures(&text) else {
        return (None, text);
    };

    (
        Some(captures[1].to_owned()),
        captures[2].to_owned()
    )
}