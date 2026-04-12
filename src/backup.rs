use std::{env, ffi::OsStr, path::Path, process::Command};

pub fn backup(vault: &Path) {
    git(["add",    "-A"],           vault);
    git(["commit", "-m", "backup"], vault);
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