#![allow(unused_imports)]
#![allow(dead_code)]

mod vault;
mod backup;

use std::{fmt::Display, hint, io::{self, Stdout, Write}};

use vault::Index;

use crate::vault::{BulkAssign, md_file::FmProperty};
use crate::vault::md_file::MdFile;


fn main() {

    let mut index = Index::build();

    index.delete_empty_unnamed_files();

    print_vault_status(&index);
    do_query          (&index);


    update_files(&mut index);
}


fn do_query(index: &Index) {

    // // TODO: figure out how to subdivide categories
    // print_all_by_category(&index, "todo");

    let files = index
        .iter_files()
        .filter    (|f| {
                f.has_property_val    (FmProperty::Category, "todo")
            && !f.has_property_val_any(FmProperty::Status,   &["completed", "archived"])

        })
    ;

    display_list_sorted_by_name("Incomplete TODO", files);

    // let files = index.md_files
    //     .iter  ()
    //     .filter(|f| f.frontmatter.processing.is_some())
    //     .map   (|f| f.as_ref())
    // ;

    // display_list_sorted_by_name("Processing", files);

}


fn update_files(index: &mut Index) {

    if !confirm_with_user("Update Files?") {
        return;
    }


    index.backup();

    println!("updating files");

    // let files = vec![
    // ];

    // index.bulk_assign_property(FmProperty::Category, "todo", |f| {
    //     files.contains(&f.file_name.as_str())
    // });


    // index.bulk_assign_property(FmProperty::Category, "todo", |f| {
    //     files.contains(&f.file_name.as_str())
    // });

    // for file in index.iter_files_mut() {
    //     if file.rename_property(FmProperty::Inbox, FmProperty::Category) {
    //         file.write_file();
    //     }

    // }



    // index.bulk_assign_processing_tag("needs_review", |f| {
    //     true
    // });

}

// ---- print status

fn print_vault_status(index: &Index) {
    print_needs_category  (&index);
    print_categories      (&index);
    print_unnamed_files   (&index);
}


fn print_needs_category(index: &Index) {
    let filenames = index.needs_category()
        .map(|f| &f.file_name)
    ;

    display_list_sorted("needs category", filenames, |f| *f);
}

fn print_categories(index: &Index) {

    let categories = index.list_all_categories();
    let width      = categories.iter().map(|x| x.0.len()).max().unwrap();

    let categories = categories
        .iter()
        .map(|x| CategoryMap(x.0, x.1.len(), width))
    ;

    display_list_sorted("Categories", categories, |i| i.0);
}

fn print_empty_files(index: &Index) {
    display_list_sorted_by_name(
        "Empty files",
        index.list_empty_unnamed_files(),
    );
}

fn print_unnamed_files(index: &Index) {
    display_list_sorted_by_name(
        "Unnamed files",
        index.list_unnamed_files(),
    );
}

fn print_all_by_category(index: &Index, name: &str) {

    let category    = index.list_all_categories();
    let Some(files) = category.get(name) else {
        println!("Category '{name}' not found");
        return;
    };

    let files = to_obsidian_list(files.iter().map(|f| *f).collect());

    println!("files:\n{}", files);
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

fn display_list_sorted_by_name<'a>(msg: &str, iter: impl Iterator<Item = &'a MdFile>) {

    let list = iter.map(|f| f.file_name.as_str());

    display_list_sorted(msg, list, |f| *f);
}

struct CategoryMap<'a>(&'a str, usize, usize);

impl<'a> Display for CategoryMap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let spaces = " ".repeat(self.2 - self.0.len());

        write!(f, "{},{} {}", self.0, spaces, self.1)
    }
}


fn to_obsidian_list(mut files: Vec<&MdFile>) -> String {

    files.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

    let mut result = String::new();
    for file in files {

        let line = format!("- [ ] [[{}]]\n", file.file_name);
        result.push_str(&line);
    }

    result
}

fn get_usr_in(prompt: &str) -> String {
    let mut buffer = String::new();
    let     stdin  = io    ::stdin ();
    let mut stdout = io    ::stdout();

    print!("{prompt}\n> ");
    stdout.flush().unwrap();

    stdin.read_line(&mut buffer).unwrap();

    buffer.trim().to_owned()

}

fn confirm_with_user(prompt: &str) -> bool {
    let prompt = format!("{prompt} (y/N)");

    let usr_in = get_usr_in(&prompt);

    usr_in.to_lowercase().starts_with("y")
}
