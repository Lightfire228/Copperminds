mod vault;

use std::{fmt::Display, iter};

use vault::Index;



fn main() {

    let index = Index::build();

    let rest = index.needs_inbox();


    let filenames = rest
        .map(|f| &f.file_name)
    ;

    display_list_sorted("needs inbox", filenames, |f| *f);

    let inboxes = index.list_all_inboxes();

    println!("");
    display_list_sorted("Inboxes", inboxes.iter(), |i| *i);

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
    where T: Display,
          K: Ord,
          F: Fn(&T) -> K,
{
    let mut list: Vec<_> = iter.collect();

    list.sort_by_key(sort_by);

    display_list(msg, list.into_iter());
}