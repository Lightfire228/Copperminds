use std::f64;
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::{collections::HashSet, env, fs, path::PathBuf};
use std::hash::Hash;

use anyhow::Ok;
use fs_extra::dir::CopyOptions;
use rand::{self, random_bool, random_range};
use yaml_serde::Mapping;
use crate::prelude::*;

use crate::vault::fm::{FmAction, GetKey};
use crate::vault::generator::gen_info::generate_info;
use crate::vault::{Env};

mod utils;
mod gen_info;

macro_rules! pick {
    ($ident:expr) => {{
        let i = rand::random_range(0..$ident.len());
        $ident.get(i).unwrap()
    }};
    [$($ident:expr,)+$(,)?] => {{
        let     max = [$($ident,)+].iter().map(|x| x.len()).sum();
        let mut choice = rand::random_range(0..max);

        let mut run = || {
            $(
                if choice < $ident.len() {
                    return *($ident.get(choice).unwrap());
                }
                choice -= $ident.len();
            )+
            unreachable!();
        };

        run()

    }};
}

struct GeneratorOpts {
    path:          PathBuf,

    file_count:    Range<usize>,
    settings:      Settings,
}

struct Settings {
    gen_info:        bool,
    gen_actionables: bool,
    gen_unnamed:     bool,
    gen_unsorted:    bool,
}



pub fn generate_sample_vault() {
    info!("Running generator");

    let env = Env::Dev;

    let dev_vault = env.vault_path();
    let folder    = dev_vault.join("01 Generated Vault");

    clear_vault(env);
    fs::create_dir(&folder).unwrap();


    let opts = GeneratorOpts {
        path:       folder,
        file_count: 3000..4000,
        settings:   Settings {
            gen_info:        true,
            gen_actionables: true,
            gen_unsorted:    true,

            gen_unnamed:     false,
        }
    };

    let max_count = random_range(opts.file_count.clone());

    let count = (0..max_count)
        .into_iter ()
        .filter_map(|_| generate_file(&opts))
        .count     ()
    ;

    write_generator_statistics(&dev_vault, count);
}



fn clear_vault(env: Env) {
    assert_ne!(env, Env::Prod, "YOU FOOL");

    let path = env.vault_path();

    fs::remove_dir_all(&path).unwrap();
    fs::create_dir    (&path).unwrap();

    let obsidian_template = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generate/.obsidian");

    fs_extra::copy_items(&[obsidian_template], path, &CopyOptions::new()).unwrap();

}



fn write_generator_statistics(dev_vault: &Path, file_count: usize) {

    macro_rules! f {
        ($($args:tt)*) => {
            format!($($args)*)
        };
    }

    let file = dev_vault.join("Generator Stats.md");
    let now  = chrono::Local::now();

    // MAYBE: can this be proc macroed like quote!{} ?
    let text = vec![
        f!("---"),
        f!("type: info"),
        f!("---"),
        f!(""),
        f!("Generated on"),
        f!("- {}", now.format("%Y-%m-%d %H:%M:%S")),
        f!(""),
        f!("Files generated"),
        f!("- {file_count}"),

    ]
        .join("\n")
    ;

    fs::write(file, text).unwrap()
}


// --- Generators

fn generate_file(opts: &GeneratorOpts) -> Option<()> {

    match pick_file_type(opts) {
        FileType::Actionable => generate_actionable(opts),
        FileType::Info       => generate_info      (opts),
    }
}

fn pick_file_type(_opts: &GeneratorOpts) -> FileType {
    match rand::random_range(0.0 .. 1.0) {
        0.00 .. 0.75 => FileType::Info,
        _            => FileType::Actionable,

    }
}


fn generate_actionable(opts: &GeneratorOpts) -> Option<()> {

    if !opts.settings.gen_actionables {
        return None;
    }


    enum ActionableState {
        None,
        Unsorted,
        Sorted,
    }


    let action = pick!(&[
        FmAction::Todo,
        FmAction::Backlog,
        FmAction::WaitingFor,
        FmAction::MaybeSomeday,
    ])
        .get_key()
    ;

    let state = match rand::random_range(0.0 .. 1.0) {
        0.00 .. 0.33 => ActionableState::None,
        0.33 .. 0.66 => ActionableState::Unsorted,
        _            => ActionableState::Sorted,
    };

    let is_sorted = matches!(state, ActionableState::Sorted);

    if !(opts.settings.gen_unsorted || is_sorted) {
        None?
    }

    let fm = match state {
        ActionableState::None     => format!(""),
        ActionableState::Unsorted => format!("type: action"),
        ActionableState::Sorted   => format!(r#"
            type: action
            action: {action}
        "#),

    };

    let fm: Option<Mapping> = yaml_serde::from_str(&fm).ok();

    let fm = match fm {
        None     => String::new(),
        Some(fm) => format!("---\n{}\n---\n", yaml_serde::to_string(&fm).unwrap()),
    };

    let title = if random_bool(0.90) {
        generate_title_todo()
    }
    else if opts.settings.gen_unnamed {
        generate_title_unique()
    }
    else {
        None?
    };

    let file = get_file_name(&opts.path, &title);

    fs::write(file, fm).unwrap();

    Some(())
}

fn generate_title_todo() -> String {
    let date = utils::generate_random_date();

    let words = vec![
        pick![VERBS,],
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

    if random_bool(0.50) {
        return generate_title_noun();
    }

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
    "Cast",
    "Ruminate",
    "Deliniate",
    "Drove",
];

const VERB_PREPS: &[&'static str] = &[
    "Riped out",
    "Condescended upon",
    "Spied on",
    "Desecrated by",
    "Destroyed by",
];

const NOUNS: &[&'static str] = &[
    "Master Seventh",
    "Xu Qing",
    "Kaladin",
    "Moash",
    "the House",
    "my Car",
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

fn get_file_name(path: &Path, title: &str) -> PathBuf {
    path.join(format!("{title}.md"))
}


#[derive(Debug, PartialEq, Eq)]
enum FileType {
    Actionable,
    Info,
}

fn lorem_ipsum() -> &'static str {
    r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore
eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt
in culpa qui officia deserunt mollit anim id est laborum.
    "#
}
