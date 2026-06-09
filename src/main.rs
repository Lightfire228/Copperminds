
mod vault;
mod backup;
mod summary;


use std::{fmt::Display, hint, io::{self, Stdout, Write}};

use vault::Index;

use crate::vault::{BulkAssign, md_file::{FmProperty, FmPropertyList}};
use crate::vault::md_file::MdFile;


fn main() {

    let mut index = Index::build();

    index.delete_empty_unnamed_files();

    // put the most important at the bottom of the terminal
    print_categories      (&index);

    write_summary_page(&mut index);
    // update_files(&mut index);
}

fn write_summary_page(index: &mut Index) {
    index.backup();

    let summary = summary::get_summary(&index);

    let file = index
        .md_files
        .iter_mut()
        .find    (|x| x.file_name == "Copperminds Summary Page.md")
        .expect  ("Unable to find Copperminds Summary Page")
    ;

    file.md_text = summary;
    file.write_file();

}
// ---- print status


fn print_categories(index: &Index) {

    let categories = index.list_all_categories();
    let width      = categories.iter().map(|x| x.0.len()).max().unwrap();

    let categories = categories
        .iter()
        .map(|x| CategoryMap(x.0, x.1.len(), width))
    ;

    display_list_sorted("Categories", categories, |i| i.0);
}


fn _print_all_by_category(index: &Index, name: &str) {

    let category    = index.list_all_categories();
    let Some(files) = category.get(name) else {
        println!("Category '{name}' not found");
        return;
    };

    let mut files: Vec<_> = files.iter().map(|f| format!("- {}", f.file_name)).collect();
    files.sort();

    println!("files:\n{}", files.join("\n"));
}


// -- utils

fn display_list<T>(msg: &str, iter: impl Iterator<Item = T>)
where
    T: Display
{
    let list: Vec<_> = iter.collect();

    if list.is_empty() {
        println!("{msg}: []");
        return;
    }

    let count = list.len();
    println!("{msg}:");

    for x in list {
        println!(" - {x}");
    }

    println!("/{msg}");
    println!("count: {count}");
    println!("");

}

fn display_list_sorted<F, T, K>(msg: &str, iter: impl Iterator<Item = T>, sort_by: F)
where
    T: Display,
    K: Ord,
    F: Fn(&T) -> K,
{
    let mut list: Vec<_> = iter.collect();

    list.sort_by_key(sort_by);

    display_list(msg, list.into_iter());
}

struct CategoryMap<'a>(&'a str, usize, usize);

impl<'a> Display for CategoryMap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let spaces = " ".repeat(self.2 - self.0.len());

        write!(f, "{},{} {}", self.0, spaces, self.1)
    }
}
