#![allow(unused_imports)]

use copperminds::*;

use pretty_env_logger::formatted_builder;

use crate::{cli::{MenuOption}, vault::ENV};

#[tokio::main]
async fn main() {

    formatted_builder()
        .filter_module("",            log::LevelFilter::Warn)
        .filter_module("wgpu_hal",    log::LevelFilter::Off)
        .filter_module("copperminds", log::LevelFilter::Trace)
        .init()
    ;

    println!("\n\n---\n");

    match ENV {
        vault::Env::Prod => {
            println!("######### ENV #########");
            println!("# Prod");
            println!("#");

            println!("\n---\n");
        },
        _ => {}
    }



    match menu() {
        Menu::SortByType       => sort_type   ::main(),
        Menu::SortByAction     => sort_actions::main(),
        Menu::ActionablesInbox => actionables ::main(),
        Menu::GenerateVault    => vault       ::generate_vault(),
        Menu::IcedUI           => ui          ::main(vault::serve()),
    }
}


#[allow(unused)]
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
            code:  "g",
            name:  "generate vault",
            value: Menu::GenerateVault,
        },
        MenuOption {
            code:  "u",
            name:  "iced ui",
            value: Menu::IcedUI,
        }
    ];

    cli::choose("Sorting method", &opts)
    // Menu::IcedUI

}

#[derive(Debug, Clone, Copy)]
enum Menu {
    SortByType,
    SortByAction,
    ActionablesInbox,
    GenerateVault,
    IcedUI,
}


#[allow(unused)]
fn __() {
    use std::path::{PathBuf};

    backup::backup(&PathBuf::from(""));
}
