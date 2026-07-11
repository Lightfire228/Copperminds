use crate::vault::{Index, md_file::{FmProperty, MdFile}};

pub fn main(index: &mut Index) {
    let files: Vec<_> = index
        .iter_files()
        .filter(|f|
            f.is_actionable()
        )
        .collect()
    ;

    let unsorted      = get(&files, |f| f.is_unactioned() && !f.is_complete_or_archived());
    let complete      = get(&files, |f| f.is_complete());
    let archived      = get(&files, |f| f.is_archived());
    let todo          = get(&files, |f| f.has_property_val(FmProperty::Action, "todo")          && !f.is_complete_or_archived());
    let waiting_for   = get(&files, |f| f.has_property_val(FmProperty::Action, "waiting_for")   && !f.is_complete_or_archived());
    let maybe_someday = get(&files, |f| f.has_property_val(FmProperty::Action, "maybe_someday") && !f.is_complete_or_archived());
    let project       = get(&files, |f| f.has_property_val(FmProperty::Action, "project")       && !f.is_complete_or_archived());

    println!("unsorted      {}", unsorted     .iter().count());
    println!("complete      {}", complete     .iter().count());
    println!("archived      {}", archived     .iter().count());
    println!("todo          {}", todo         .iter().count());
    println!("waiting_for   {}", waiting_for  .iter().count());
    println!("maybe_someday {}", maybe_someday.iter().count());
    println!("project       {}", project      .iter().count());
}

fn get<'a, F>(files: &[&'a MdFile], f: F) -> Vec<&'a MdFile>
where
    F: Fn(&&&'a MdFile) -> bool
{
    files
        .iter   ()
        .filter (f)
        .map    (|f| *f)
        .collect()

}
