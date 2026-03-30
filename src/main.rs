use std::{io, process::{Command, Output}};


mod yaml;
mod obsidian;

fn main() {
    println!("Hello, world!");

    yaml::scan("");

    let files = obsidian::get_all_md_files();

    obsidian::get_all_inboxes(&files);

}

