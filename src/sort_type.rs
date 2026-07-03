
use std::{process::Command};

use crate::{cli::{MenuOption, choose}, vault::{Index, md_file::{FmProperty, MdFile}}};


pub fn main(index: &mut Index) {

    let action_count = index.iter_files().filter(|t| t.has_property_val(FmProperty::Type, "action")).count();

    let files: Vec<_> = index
        .iter_files_mut()
        .filter        (|f| f.is_untyped())
        .collect       ()
    ;

    println!("remaining:     {}", files.len());
    println!("count: action: {}", action_count);


    for file in files {

        display_file(file);

        let type_ = get_type();

        // re-load any changes made to the file while the cli was waiting for input
        // NOTE: this doesn't catch renames or deletes
        file.refresh();

        file.set_type(type_);
        file.write_file();
    }
}

fn display_file(file: &MdFile) {
    println!("\n\n");
    println!("=== {} ===", file.file_name);
    // println!("{}",         file.raw_text)

    open_in_obsidian(file);
}


fn get_type() -> Type {
    let opts = [
        MenuOption {
            code:  "i",
            name:  "info",
            value: Type::Info,
        },
        MenuOption {
            code:  "a",
            name:  "action",
            value: Type::Action,
        },
    ];

    choose("Type", &opts)

}

#[derive(Debug, Clone, Copy)]
enum Type {
    Info,
    Action,
}

impl MdFile {
    fn set_type(&mut self, type_: Type) {
        self.assign_property(FmProperty::Type, match type_ {
            Type::Info   => "info"  .to_owned(),
            Type::Action => "action".to_owned(),
        });
    }
}


fn open_in_obsidian(file: &MdFile) {

    let uri = format!("obsidian://open?vault=Notes&file={}", urlencoding::encode(&file.file_name));

    Command::new("xdg-open")
        .arg   (uri)
        .output()
        .unwrap()
    ;
}
