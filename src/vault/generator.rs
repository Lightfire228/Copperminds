use std::io::Write;
use std::path::Path;
use std::{collections::HashSet, env, fs, path::PathBuf};
use std::hash::Hash;

use fs_extra::dir::CopyOptions;
use rand::{self, random_bool};
use crate::prelude::*;

use crate::vault::{Env};

mod utils;

macro_rules! pick {
    ($ident:ident) => {{
        let i = rand::random_range(0..$ident.len());
        $ident[i]
    }};
    [$($ident:ident,)+$(,)?] => {{
        let     max = [$($ident,)+].iter().map(|x| x.len()).sum();
        let mut choice = rand::random_range(0..max);

        let mut run = || {
            $(
                if choice < $ident.len() {
                    return $ident[choice];
                }
                choice -= $ident.len();
            )+
            unreachable!();
        };

        run()

    }};
}


pub fn generate_sample_vault() {
    info!("Running generator");

    let env = Env::Dev;

    let dev_vault = env.vault_path();
    let folder    = dev_vault.join("01 Generated Vault");

    clear_vault(env);
    fs::create_dir(&folder).unwrap();


    for file in generate_sample_vault_titles() {

        let path = folder.join(file.title());

        fs::File::create(&path).unwrap();

        write_data(&file, &path);
    }

    write_generator_statistics(&dev_vault);


}

fn write_data(file: &File, path: &Path) {

    macro_rules! with_chance {
        ($chance:literal, $expr:expr) => {

            if random_bool($chance) {

                fs::write(&path, $expr)
                    .unwrap()
                ;
            }
        };
    }

    match &file.kind {
        FileType::Todo    => {
            if random_bool(0.33) {
                fs::write(&path, "---\ntype: action\n---\n").unwrap();
            }
            else if random_bool(0.33) {
                fs::write(&path, "---\ntype: action\naction: todo\n---\n").unwrap();
            }
            else {
                fs::write(&path, "---\ntype: action\naction: todo\nstatus: complete\n---\n").unwrap();
            }
        },
        FileType::Info    => with_chance!(0.50, "---\ntype: info\n---\n"),
        FileType::Unnamed => with_chance!(0.90, "not empty\n"),
    }
}



fn clear_vault(env: Env) {
    assert_ne!(env, Env::Prod, "YOU FOOL");

    let path = env.vault_path();

    fs::remove_dir_all(&path).unwrap();
    fs::create_dir    (&path).unwrap();

    let obsidian_template = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generate/.obsidian");

    fs_extra::copy_items(&[obsidian_template], path, &CopyOptions::new()).unwrap();

}



fn write_generator_statistics(dev_vault: &Path) {

    macro_rules! f {
        ($($args:tt)*) => {
            format!($($args)*)
        };
    }

    let file = dev_vault.join("Generator Stats.md");
    let now  = chrono::Local::now();

    // MAYBE: can this be proc macroed like quote!{} ?
    let text = vec![
        f!("Generated on"),
        f!("- {}", now.format("%Y-%m-%d %H:%M:%S")),
        f!(""),
    ]
        .join("\n")
    ;

    fs::write(file, text).unwrap()
}


// --- Generators

fn generate_sample_vault_titles() -> HashSet<File> {

    (0..3000)
        .into_iter()
        .map      (|_| generate_title())
        .collect  ()

}

fn generate_title() -> File {
    match rand::random_range(0.0 .. 1.0) {
        0.00 .. 0.25 => File { title: generate_title_noun(),   kind: FileType::Info},
        0.25 .. 0.50 => File { title: generate_title_unique(), kind: FileType::Unnamed},
        0.50 .. 0.75 => File { title: generate_title_info(),   kind: FileType::Info},
        _            => File { title: generate_title_todo(),   kind: FileType::Todo},
    }
}

fn generate_title_todo() -> String {
    let date = utils::generate_random_date();

    let words = vec![
        pick![VERBS],
        pick!(PARTICLES),
        pick!(NOUNS),
        pick!(PREPOSITIONS),
        pick!(PARTICLES),
        pick!(NOUNS),
    ];

    let title = words.join(" ");

    format!("{date} - TODO - {title}")
}

fn generate_title_info() -> String {
    let date = utils::generate_random_date();

    let words = vec![
        pick![VERBS, VERB_PREPS,],
        pick!(PARTICLES),
        pick!(NOUNS),
        pick!(PREPOSITIONS),
        pick!(PARTICLES),
        pick!(NOUNS),
        pick!(WHEN),
    ];

    let title = words.join(" ");

    format!("{date} - {title}")
}

fn generate_title_noun() -> String {
    // the number is for uniqueness
    format!("{} - {}", pick!(NOUNS), rand::random_range(0..1000))

}

fn generate_title_unique() -> String {
    let date = utils::generate_random_date();
    let time = utils::generate_random_time();

    format!("{date} - {time}")
}


const VERBS: &[&'static str] = &[
    "Throngle",
    "Glorneate",
    "Desecrate",
    "Cultivate",
    "Cast",
    "Ruminate",
    "Deliniate",
];

const VERB_PREPS: &[&'static str] = &[
    "Riped out",
    "Condescended upon",
    "Spied on",
    "Desecrated by",
    "Destroyed by",
];

const NOUNS: &[&'static str] = &[
    "Nascent Soul",
    "Gold Core",
    "County capital",
    "senior apprentice brother",
    "Master Seventh",
    "Xu Qing",
    "Meng Hao",
    "Kaladin",
    "Moash",
];

const PARTICLES: &[&'static str] = &[
    "a",
    "the",
    "one",
    "two",
    "many",
    "few",
];

const PREPOSITIONS: &[&'static str] = &[
    "on",
    "upon",
    "between",
    "in",
    "inside",
    "outside",
    "under",
    "over",
    "at",
];

const WHEN: &[&'static str] = &[
    "just now",
    "now",
    "today",
    "this week",
    "a moment ago",
];


#[derive(Debug, PartialEq, Eq)]
struct File {
    title: String,
    kind:  FileType,
}

#[derive(Debug, PartialEq, Eq)]
enum FileType {
    Todo,
    Info,
    Unnamed,
}

impl Hash for File {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
    }
}


impl File {
    pub fn title(&self) -> String {
        format!("{}.md", self.title)
    }
}
