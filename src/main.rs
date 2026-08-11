
mod actionables;
mod backup;
mod cli;
mod obsidian;
mod sort_actions;
mod sort_type;
mod vault;
mod ui;



use crate::{cli::{MenuOption, choose}};

#[tokio::main]
async fn main() {

    println!("\n\n---\n");

    match menu() {
        Menu::SortByType       => sort_type   ::main(),
        Menu::SortByAction     => sort_actions::main(),
        Menu::ActionablesInbox => actionables ::main(),
        Menu::IcedUI           => ui          ::main(vault::serve()),
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
        },
        MenuOption {
            code:  "u",
            name:  "iced ui",
            value: Menu::IcedUI,
        }
    ];

    choose("Sorting method", &opts)
}

#[derive(Debug, Clone, Copy)]
enum Menu {
    SortByType,
    SortByAction,
    ActionablesInbox,
    IcedUI,
}
