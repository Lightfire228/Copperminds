use std::{collections::HashSet, env, fs, path::PathBuf, process::Command};

use fs_extra::dir::CopyOptions;
use futures::stream::iter;
use rand::{self, random_bool};

use crate::vault::{ENV, Env};

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
    let env = Env::Dev;

    let dev_vault = env.vault_path();
    let folder    = dev_vault.join("01 Generated Vault");

    clear_vault(env);
    fs::create_dir(&folder).unwrap();


    for title in generate_sample_vault_titles() {

        let path = folder.join(&title);

        fs::File::create(&path).unwrap();

        if !title.contains("TODO") {
            continue;
        }

        if random_bool(0.50) {
            continue;
        }

        fs::write(path, "---\ntype: action\n---\n")
            .unwrap()
        ;

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


pub fn generate_sample_vault_titles() -> HashSet<String> {

    (0..3000)
        .into_iter()
        .map      (|_| format!("{}.md", generate_title()))
        .collect  ()

}

fn generate_title() -> String {
    match rand::random_range(0.0 .. 1.0) {
        0.00 .. 0.25 => generate_title_noun(),
        0.25 .. 0.50 => generate_title_unique(),
        0.50 .. 0.75 => generate_title_info(),
        _            => generate_title_todo(),
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
