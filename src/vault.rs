use std::{env, fs, path::PathBuf};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use yaml_serde::Mapping;


pub struct Index {
    pub md_files: Vec<MdFile>,
    
}

pub struct MdFile {
    pub entry:       DirEntry,
    pub frontmatter: Option<Mapping>,
    pub inbox:       Option<String>,
}

impl Index {
    pub fn build() -> Self {
        let files    = scan_vault();

        let md_files = md_files(&files);

        let re       = frontmatter_regex();

        let md_files = md_files
            .iter     ()
            .map      (|f| parse_md_file(&re, &f))
            .collect  ()
        ;



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


fn parse_md_file(re: &Regex, file: &DirEntry) -> MdFile {

    
    let blank = || {
        MdFile {
            entry:       file.clone(),
            frontmatter: None,
            inbox:       None,
        }
    };

    let Some(text) = get_frontmatter_text(&re, &file) else {
        return blank();
    };

    let Some(fm)   = yaml_serde::from_str::<Mapping>(&text).ok() else {
        return blank();
    };

    MdFile {
        entry:       file.clone(),
        inbox:       fm
            .get     ("inbox")
            .and_then(|i| i.as_str())
            .map     (|i| i.to_owned())
        ,
        frontmatter: Some(fm),
    }

}


fn get_frontmatter_text(re: &Regex, file: &DirEntry) -> Option<String> {

    let text     = fs::read_to_string(file.path()).unwrap();
    
    let captures = re.captures(&text)?;
    
    Some(captures[1].to_owned())
}

fn frontmatter_regex() -> Regex {
    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    Regex::new(r"(?ms)^---\s*(.+?)^---\s*$(.*)").unwrap()
}