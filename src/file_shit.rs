use std::{fs, path::Path};


/// Includes the file extension
pub fn get_file_name(path: &Path) -> String {
    path
        .file_name()
        .unwrap   ()
        .to_str   ()
        .unwrap   ()
        .to_owned ()
}

pub fn get_file_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn delete_from_disk(path: &Path) {
    trash::delete(path).unwrap();
}
