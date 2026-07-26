use crate::vault::{Index, md_file::{FmProperty}};

pub fn main(index: &mut Index) {
    let files: Vec<_> = index
        .iter_files()
        .filter(|f|
            f.is_actionable()
        )
        .collect()
    ;

    let mut unsorted      = vec![];
    let mut complete      = vec![];
    let mut archived      = vec![];
    let mut todo          = vec![];
    let mut waiting_for   = vec![];
    let mut maybe_someday = vec![];
    let mut project       = vec![];
    let mut unknown       = vec![];

    for file in files {

        let bucket = match () {
            _ if file.is_complete  ()                                       => &mut complete,
            _ if file.is_archived  ()                                       => &mut archived,
            _ if file.is_unactioned()                                       => &mut unsorted,
            _ if file.has_property_val(FmProperty::Action, "todo")          => &mut todo,
            _ if file.has_property_val(FmProperty::Action, "waiting_for")   => &mut waiting_for,
            _ if file.has_property_val(FmProperty::Action, "maybe_someday") => &mut maybe_someday,
            _ if file.has_property_val(FmProperty::Action, "project")       => &mut project,
            _ => &mut unknown,
        };

        bucket.push(file);
    }

    println!("unsorted      {}", unsorted     .iter().count());
    println!("complete      {}", complete     .iter().count());
    println!("archived      {}", archived     .iter().count());
    println!("todo          {}", todo         .iter().count());
    println!("waiting_for   {}", waiting_for  .iter().count());
    println!("maybe_someday {}", maybe_someday.iter().count());
    println!("project       {}", project      .iter().count());
}
