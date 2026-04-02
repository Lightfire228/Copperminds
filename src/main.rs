use std::{env, fs, iter, path::{Path, PathBuf}};

use regex::Regex;
use walkdir::{DirEntry, IntoIter, WalkDir};
use yaml_serde::Mapping;

use crate::vault::Index;

mod vault;



fn main() {

    let index = Index::build();

    for f in index.md_files {

        let Some(text) = get_frontmatter_text(&f) else {
            continue;
        };

        parse_frontmatter(&text);


    }

}


fn get_frontmatter_text(file: &DirEntry) -> Option<String> {

    // TODO: https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop

    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    let re       = Regex::new(r"(?ms)^---\s*(.+?)^---\s*$(.*)").unwrap();

    let text     = fs::read_to_string(file.path()).unwrap();

    let captures = re.captures(&text)?;

    Some(captures[1].to_owned())
}

fn parse_frontmatter(text: &str) -> Option<Mapping> {
    yaml_serde::from_str::<Mapping>(text).ok()
}