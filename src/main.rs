
mod backup;
mod cli;
mod obsidian;
mod sort_actions;
mod sort_type;
mod vault;



use vault::Index;

use crate::cli::{MenuOption, choose};


fn main() {

    let mut index = Index::build();

    index.delete_empty_unnamed_files();

    println!("\n\n---\n");

    match menu() {
        Sort::ByType   => sort_type   ::main(&mut index),
        Sort::ByAction => sort_actions::main(&mut index),
    }
}


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

    choose("Sorting method", &opts)

}

#[derive(Debug, Clone, Copy)]
enum Sort {
    ByType,
    ByAction,

}
