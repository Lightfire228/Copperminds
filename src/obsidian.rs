use std::process::Command;

use crate::vault::md_file::MdFile;

use crate::vault::ENV;

pub fn open_in_obsidian(file: &MdFile) {

    let vault = ENV.vault_name();

    let uri = format!("obsidian://open?vault={vault}&file={}", urlencoding::encode(&file.file_name));

    Command::new("xdg-open")
        .arg   (uri)
        .output()
        .unwrap()
    ;
}
