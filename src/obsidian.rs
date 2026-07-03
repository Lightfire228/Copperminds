use std::process::Command;

use crate::vault::md_file::MdFile;


pub fn open_in_obsidian(file: &MdFile) {

    let uri = format!("obsidian://open?vault=Notes&file={}", urlencoding::encode(&file.file_name));

    Command::new("xdg-open")
        .arg   (uri)
        .output()
        .unwrap()
    ;
}
