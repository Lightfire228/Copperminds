use std::process::Command;

use crate::vault::{ENV, EcsFileView};

pub fn open_in_obsidian(file_name: &str) {

    let vault = ENV.vault_name();

    let uri = format!("obsidian://open?vault={vault}&file={}", urlencoding::encode(file_name));

    tokio::spawn(async {
        Command::new("xdg-open")
            .arg   (uri)
            .output()
            .unwrap()
        ;
    });
}
