use std::fs;

use regex::Regex;
use walkdir::{DirEntry};
use yaml_serde::Mapping;

use crate::vault::Index;

mod vault;



fn main() {

    let index = Index::build();


    for file in index.md_files {
        file
            .inbox
            .and_then(|i| {
                println!("file:  {:?}", file.entry.file_name());
                println!("inbox: {i}");

                Some(())
            })
        ;
    }
}
