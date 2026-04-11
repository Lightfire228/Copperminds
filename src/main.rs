mod vault;

use std::{fmt::Display};

use vault::Index;



fn main() {

    let index = Index::build();

    let rest = index.needs_inbox();


    let filenames = rest
        .map(|f| &f.file_name)
    ;

    display_list_sorted("needs inbox", filenames, |f| *f);

    
    let inboxes = index.list_all_inboxes();
    let width   = inboxes.iter().map(|x| x.0.len()).max().unwrap();

    let inboxes = inboxes
        .iter()
        .map(|x| InboxMap(x.0, x.1.len(), width))
    ;

    println!("");
    display_list_sorted("Inboxes", inboxes, |i| i.0);

    // index.bulk_assign_inbox_by_name("other", |f| {
    //     true
    // });
}


fn display_list<T>(msg: &str, iter: impl Iterator<Item = T>)
    where T: Display
{
    let mut count = 0;

    println!("{msg}:");

    for x in iter {
        println!(" - {x}");
        count += 1;
    }

    println!("/{msg}");
    println!("count: {count}");
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

struct InboxMap<'a>(&'a str, usize, usize);

impl<'a> Display for InboxMap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let spaces = " ".repeat(self.2 - self.0.len());
        write!(f, "{},{} {}", self.0, spaces, self.1)
    }
}