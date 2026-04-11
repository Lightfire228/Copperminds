mod vault;

use std::{fmt::Display};

use vault::Index;

use crate::vault::MdFile;


fn main() {

    let mut index = Index::build();

    print_needs_inbox  (&index);
    print_inboxes      (&index);
    print_unnamed_files(&index);
    print_empty_files  (&index);

    index.delete_empty_files();
    
    // index.bulk_assign_inbox_by_name("other", |f| {
    //     true
    // });
}


fn print_needs_inbox(index: &Index) {
    let filenames = index.needs_inbox()
        .map(|f| &f.file_name)
    ;

    display_list_sorted("needs inbox", filenames, |f| *f);
}

fn print_inboxes(index: &Index) {

    let inboxes = index.list_all_inboxes();
    let width   = inboxes.iter().map(|x| x.0.len()).max().unwrap();

    let inboxes = inboxes
        .iter()
        .map(|x| InboxMap(x.0, x.1.len(), width))
    ;

    display_list_sorted("Inboxes", inboxes, |i| i.0);
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




fn display_list<T>(msg: &str, iter: impl Iterator<Item = T>)
where 
    T: Display
{
    let list: Vec<_> = iter.collect();

    if list.is_empty() {
        println!("{msg}: []");
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

struct InboxMap<'a>(&'a str, usize, usize);

impl<'a> Display for InboxMap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let spaces = " ".repeat(self.2 - self.0.len());

        write!(f, "{},{} {}", self.0, spaces, self.1)
    }
}