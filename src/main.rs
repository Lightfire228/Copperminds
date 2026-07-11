
mod actionables;
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
        Menu::SortByType       => sort_type   ::main(&mut index),
        Menu::SortByAction     => sort_actions::main(&mut index),
        Menu::ActionablesInbox => actionables ::main(&mut index),
    }
}


fn menu() -> Menu {
    let opts = [
        MenuOption {
            code:  "t",
            name:  "sort by Type",
            value: Menu::SortByType,
        },
        MenuOption {
            code:  "a",
            name:  "sort by Action",
            value: Menu::SortByAction,
        },
        MenuOption {
            code:  "ai",
            name:  "actionables inbox",
            value: Menu::ActionablesInbox,
        }
    ];

    choose("Sorting method", &opts)
}

#[derive(Debug, Clone, Copy)]
enum Menu {
    SortByType,
    SortByAction,
    ActionablesInbox,
}
