use std::{env, fs, path::{Path, PathBuf}};

use fancy_regex::Captures;

use crate::vault::build_regex;


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

pub fn home_dir() -> PathBuf {
    env::home_dir().expect("unable to get home dir")
}

pub fn resolve_tilde(path: PathBuf) -> PathBuf {
    build_regex!(fancy RE = r"(?<!\\)~");

    let path_str = path.to_str().unwrap();

    let captures = RE
        .captures_iter(path_str)
        .collect      ::<Result<Vec<_>, _>>()
        .expect       ("error running regex")
    ;

    if captures.is_empty() {
        return path;
    }

    let replaced = replace_tilde(path_str, captures);

    PathBuf::from(replaced)
}

fn replace_tilde(path_str: &str, captures: Vec<Captures<'_, str>>) -> String {

    let mut replaced = String::new();

    let home = home_dir();
    let home = home.to_str().unwrap();

    let mut replace = |start, end| {
        replaced.push_str(&path_str[start..end]);
        replaced.push_str(home);
    };


    let mut captures = captures
        .into_iter()
        .map(|x| x
            .get   (0)
            .expect("error running regex")
        )
    ;

    let first = captures.next().unwrap();
    replace(0, first.start());

    let mut start = first.end();

    for x in captures {
        replace(start, x.start());

        start = x.end();
    }

    replaced.push_str(&path_str[start..path_str.len()]);

    replaced

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tilde() {

        let home = home_dir();
        let home = home.to_str().unwrap();

        struct Test {
            path:    String,
            resolve: String,
        }

        let tests = vec![
            Test {
                path:    format!("none/"),
                resolve: format!("none/"),
            },
            Test {
                path:    format!("~"),
                resolve: format!("{home}"),
            },
            Test {
                path:    format!("~/folder"),
                resolve: format!("{home}/folder"),
            },
            Test {
                path:    format!(r"\~"),
                resolve: format!(r"\~"),
            },
            Test {
                path:    format!("~/../../~/../~~"),
                resolve: format!("{home}/../../{home}/../{home}{home}"),
            },
        ];


        for x in tests {
            assert_eq!(resolve_tilde(PathBuf::from(x.path)), PathBuf::from(x.resolve));
        }
    }

}
