use crate::vault::{Index, fm::{FmAction, FmProperty}};


pub fn main(index: &mut Index) {
    let files: Vec<_> = index
        .iter_files_with(|f|
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

    for id in files {

        let file = index.get_file_mut(id);

        let bucket = match () {
            _ if file.is_complete      ()                                     => &mut complete,
            _ if file.is_archived      ()                                     => &mut archived,
            _ if file.needs_action_type()                                     => &mut unsorted,
            _ if file.is_property(FmProperty::Action, FmAction::Todo)         => &mut todo,
            _ if file.is_property(FmProperty::Action, FmAction::WaitingFor)   => &mut waiting_for,
            _ if file.is_property(FmProperty::Action, FmAction::MaybeSomeday) => &mut maybe_someday,
            _ if file.is_property(FmProperty::Action, FmAction::Project)      => &mut project,
            _ => &mut unknown,
        };

        bucket.push(id);
    }

    println!("unsorted      {}", unsorted     .iter().count());
    println!("complete      {}", complete     .iter().count());
    println!("archived      {}", archived     .iter().count());
    println!("todo          {}", todo         .iter().count());
    println!("waiting_for   {}", waiting_for  .iter().count());
    println!("maybe_someday {}", maybe_someday.iter().count());
    println!("project       {}", project      .iter().count());
}
