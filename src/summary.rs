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

    macro_rules! iter {
        (|$ident:ident| $filter:expr) => {
            index.iter_files().filter(|$ident| $filter)
        };
    }

    // let date = vec![
    //     // Local::now().date().
    // ]
    //

    let s = |x| [String::from(x)];

    let date = [
        Local::now().format("%Y-%m-%d").to_string(),
        "".to_string(),
        "".to_string(),
    ];

    flat_vec![
        date,
        get_section("Unnamed Files:", iter!(|f| f.is_unnamed())),

        s("# Getting Things Done"),
        s(""),

        s("## Reference:"),
        s("Types:"),
        format_list([
            "info",
            "action",
        ]),
        s("Contexts:"),
        format_list([
            "todo",
            "waiting_for",
            "calendar",
            "someday",
        ]),
        s("Statuses:"),
        format_list([
            "completed",
            "archived",
        ]),
        s(""),

        get_section("Needs Type:",   iter!(|f| f.is_untyped())),
        get_section("Needs Action:", iter!(|f| f.is_unactioned())),


        s("# Deprecated"),
        s(""),
        get_section("Needs Categorized:", iter!(|f| f.is_uncategorized())),
        get_section("Incomplete Todos:",  iter!(|f| f.is_unnamed())),
    ]
        .join("\n")

}


fn get_section<'a, T>(title: &str, files: T) -> SummaryData
where
    T: Iterator<Item = &'a MdFile>
{

    flat_vec![
        [title.to_owned()],
        format_list_wikilink(files),
        [
            String::new(),
            String::new(),
        ]
    ]
}


fn format_list_wikilink<'a, T>(files: T) -> SummaryData
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

fn format_list<'a, T>(files: T) -> SummaryData
where
    T: IntoIterator<Item = &'a str>
{
    files
        .into_iter()
        .map(|f| {
            format!("- {}", f)
        })
        .collect()
}
