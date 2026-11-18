use std::process::Command;

use crate::prelude::*;

use crate::config::Config;


pub fn open_in_obsidian(config: &Config, file_name: &str) {

    trace!("open file in obsidian: {file_name}");

    let vault = &config.vault_name;

    let uri = format!("obsidian://open?vault={vault}&file={}", urlencoding::encode(file_name));

    tokio::spawn(async {
        Command::new("xdg-open")
            .arg   (uri)
            .output()
            .unwrap()
        ;
    });
}
