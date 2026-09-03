#![allow(unused_imports)]

use copperminds::*;

use pretty_env_logger::{formatted_builder, formatted_timed_builder};

use crate::{cli::{MenuOption}, vault::ENV};



#[tokio::main]
async fn main() {

    let log_level = log::LevelFilter::Info;
    // let log_level = log::LevelFilter::Trace;

    formatted_timed_builder()
        .filter_module("",            log::LevelFilter::Warn)
        .filter_module("wgpu_hal",    log::LevelFilter::Off)
        .filter_module("copperminds", log_level)
        // .filter_module("copperminds", log::LevelFilter::Info)
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

    println!("Log Level: {log_level:?}\n");


    match menu() {
        Menu::GenerateVault    => vault       ::generate_vault(),
        Menu::IcedUI           => ui          ::main(vault::serve()),
    }
}


#[allow(unused)]
fn menu() -> Menu {

    let opts = [
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
    // Menu::GenerateVault

}

#[derive(Debug, Clone, Copy)]
enum Menu {
    GenerateVault,
    IcedUI,
}


#[allow(unused)]
fn __() {
    use std::path::{PathBuf};

    backup::backup(&PathBuf::from(""));
}
