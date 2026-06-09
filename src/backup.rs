use std::{ffi::OsStr, path::Path, process::{Command, Output, Stdio}};

pub fn backup(vault: &Path) {
    backup_named(vault, "backup")
}

pub fn backup_named(vault: &Path, commit_msg: &str) {

    let branch = git(["symbolic-ref", "--short", "HEAD"], vault);
    let branch = String::from_utf8(branch.stdout).unwrap();
    let branch = branch.trim();

    git(["add",    "-A"],             vault);
    git(["commit", "-m", commit_msg], vault);
    git(["push",   "backup", branch], vault);
}

fn git<I, S>(args: I, vault: &Path) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{

    Command::new("git")
        .args       (args)
        .current_dir(vault)
        .stderr     (Stdio::inherit())
        .stdout     (Stdio::piped())
        .output     ()
        .unwrap     ()
}
