use std::fs;

use regex::Regex;
use walkdir::{DirEntry};
use yaml_serde::Mapping;

use crate::vault::Index;

mod vault;



fn main() {

    let index = Index::build();


    for file in index.needs_inbox() {
        println!("needs inbox: {:?}", file.entry.file_name());
    }
}
