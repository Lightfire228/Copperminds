use std::{env, ffi::OsStr, path::Path, process::Command};

pub fn backup(vault: &Path) {
    backup_named(vault, "backup")
}

pub fn backup_named(vault: &Path, commit_msg: &str) {
    git(["add",    "-A"],             vault);
    git(["commit", "-m", commit_msg], vault);
}

fn git<I, S>(args: I, vault: &Path)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{

    Command::new("git")
        .args       (args)
        .current_dir(vault)
        .output     ()
        .unwrap     ()
    ;
}
