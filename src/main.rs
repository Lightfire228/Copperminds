use copperminds::{vault::Env, *};


use pretty_env_logger::{formatted_timed_builder};

use crate::{cli::MenuOption};


#[tokio::main]
async fn main() {

    let env = Env::Prod;

    // let log_level = log::LevelFilter::Info;
    let log_level = log::LevelFilter::Trace;

    formatted_timed_builder()
        .filter_module("",            log::LevelFilter::Warn)
        .filter_module("wgpu_hal",    log::LevelFilter::Off)
        .filter_module("copperminds", log_level)
        // .filter_module("copperminds", log::LevelFilter::Info)
        .init()
    ;

    let config = config::get_config(env);


    println!("\n\n---\n");

    match env {
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
        Menu::IcedUI           => ui          ::main(vault::serve(&config), &config),
    }
}


fn menu() -> Menu {

    #[allow(unused)] // reason: main menu toggle
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

    // cli::choose("Sorting method", &opts)
    Menu::IcedUI
    // Menu::GenerateVault
}

#[derive(Debug, Clone, Copy)]
enum Menu {
    GenerateVault,
    IcedUI,
}
