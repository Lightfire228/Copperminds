
mod backup;
mod cli;
mod obsidian;
mod sort_actions;
mod sort_type;
mod summary;
mod vault;



use vault::Index;

use crate::cli::{MenuOption, choose};


fn main() {

    let mut index = Index::build();

    index.delete_empty_unnamed_files();


    write_summary_page(&mut index);

    match menu() {
        Sort::ByType   => sort_type   ::main(&mut index),
        Sort::ByAction => sort_actions::main(&mut index),
    }
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


fn menu() -> Sort {
    let opts = [
        MenuOption {
            code:  "t",
            name:  "sort by Type",
            value: Sort::ByType,
        },
        MenuOption {
            code:  "a",
            name:  "sort by Action",
            value: Sort::ByAction,
        }
    ];

    choose("", &opts)

}

#[derive(Debug, Clone, Copy)]
enum Sort {
    ByType,
    ByAction,

}
