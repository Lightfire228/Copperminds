use std::{env, iter, path::{Path, PathBuf}};

use walkdir::{DirEntry, IntoIter, WalkDir};

use crate::vault::Index;

mod vault;



fn main() {

    let index = Index::build();

    for f in index.md_files {
        println!("{:?}", f.file_name());
    }

}

