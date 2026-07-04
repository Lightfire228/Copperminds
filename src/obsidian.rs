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
type Lines = Vec<String>;

#[allow(dead_code)]
pub fn format_list_wikilink<'a, T>(files: T) -> Lines
where
    T: Iterator<Item = &'a MdFile>
{
    let mut files: Vec<_> = files.collect();

    files.sort_by_key(|x| &x.file_name);

    files
        .into_iter()
        .map(|f| {
            format!("- [ ] [[{}]]", f.file_name)
        })
        .collect()
}

#[allow(dead_code)]
pub fn format_list<'a, T>(files: T) -> Lines
where
    T: IntoIterator<Item = &'a str>
{
    files
        .into_iter()
        .map(|f| {
            format!("- {}", f)
        })
        .collect()
}
