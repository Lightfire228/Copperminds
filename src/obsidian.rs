use std::{io, process::{Command, Output}};

pub fn get_all_md_files() -> Vec<String> {
    let out   = obsidian_cmd("files");
    let files = out.lines();

    files
        .filter (|f| f.ends_with(".md"))
        .map    (|f| String::from(f))
        .collect()
}

pub fn get_all_inboxes(files: &[String]) -> Vec<String> {

    for f in files {
        // let name  = format!("name=inbox");
        // let file  = format!("file={f}");
        // let inbox = obsidian_cmds(&["property:read", &name, &file]);

        let inbox = get_properties(f);
        // println!("s {}", inbox);
    }

    todo!()
}

pub fn get_properties(file: &str) -> Option<()>{
    let file  = format!("file={file}");
    let inbox = obsidian_cmds(&["properties", &file, "format=tsv"]);

    if inbox.contains("No frontmatter found") {
        return None;
    }
    println!("props {}", inbox);

    Some(())

}

fn obsidian_cmd(cmd: &str) -> String {
    obsidian_cmds(&[cmd])
}

fn obsidian_cmds(cmd: &[&str]) -> String {
    let out = Command::new("/usr/bin/obsidian")
        .args  (cmd)
        .output()
        .expect("Failed to run obsidian command")
    ;

    if !out.stderr.is_empty() {
        let msg = String::from_utf8(out.stderr).expect("Unable to convert stderr to string");
        panic!("Obsidian command error: {msg}");
    }

    let out = String::from_utf8(out.stdout).expect("Unable to convert stdout to string");

    out
}