mod vault;

use vault::Index;
use walkdir::DirEntry;
use std::sync::LazyLock;
use regex::Regex;

use crate::vault::MdFile;
use crate::vault::regex;



fn main() {

    let mut index = Index::build();

    let mut rest: Vec<_> = index.needs_inbox().collect();

    rest.sort_by_key(|f| &f.file_name);

    for file in rest.iter() {
        println!("needs inbox: {}", file.file_name);
    }

    println!("count: {}", rest.len());

    index.bulk_assign_inbox_by_name("other", |f| {
        true
    });
}
