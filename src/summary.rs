use crate::vault::{Index, md_file::MdFile};

use chrono::{self, Local};

type SummaryData = Vec<String>;

macro_rules! flat_vec {
    [ $( $x:expr ),*$(,)? ] => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.extend($x);
            )*
            temp_vec
        }
    };
}

pub fn get_summary(index: &Index) -> String {

    // let date = vec![
    //     // Local::now().date().
    // ]

    let date = [
        Local::now().format("%Y-%m-%d").to_string(),
        "".to_string(),
        "".to_string(),
    ];

    let unnamed_files     = get_section("Unnamed Files:",     index.list_unnamed_files());
    let needs_categorized = get_section("Needs Categorized:", index.needs_category());
    let incomplete_todos  = get_section("Incomplete Todos:",  index.list_incomplete_todos());

    flat_vec![
        date,
        unnamed_files,
        needs_categorized,
        incomplete_todos,
    ]
        .join("\n")

}


fn get_section<'a, T>(title: &str, files: T) -> SummaryData
where
    T: Iterator<Item = &'a MdFile>
{

    flat_vec![
        [title.to_owned()],
        format_list(files),
        [
            String::new(),
            String::new(),
        ]
    ]
}

fn format_list<'a, T>(files: T) -> SummaryData
where
    T: Iterator<Item = &'a MdFile>
{
    let mut files: Vec<_> = files.collect();

    files.sort_by_key(|x| &x.file_name);

    files
        .into_iter()
        .map(|f| {
            format!("- [ ] [[{}]]", f.file_name)
        })
        .collect()
}
