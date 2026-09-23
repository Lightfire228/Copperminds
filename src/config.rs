use std::{env, fs, path::PathBuf};
use serde::{Deserialize, Serialize};


#[allow(dead_code)] // reason: prod select via const in main
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Prod,
    Dev,
}

impl Env {

    pub fn name(&self) -> &'static str {
        match self {
            Self::Prod => "Prod",
            Self::Dev  => "Dev",
        }
    }
}


use crate::{file_shit};

pub fn get_config(env: Env) -> Config {

    let file = get_config_file(env);

    let Ok(text) = fs::read_to_string(&file) else {
        panic!("unable to read config file: {file:?}")
    };

    let mut config: YamlConfig = yaml_serde::from_str(&text)
        .inspect_err(|err| panic!("unable to parse config file: {err}"))
        .unwrap()
    ;

    config.vault_folder = file_shit::resolve_tilde(config.vault_folder);

    if !config.vault_folder.exists() {
        panic!("vault folder does not exist");
    }

    Config {
        env,
        folder_excludes: config.folder_excludes,
        vault_path:      config.vault_folder,
        vault_name:      config.vault_name,
    }

}

pub fn get_config_file(env: Env) -> PathBuf {
    match env {
        Env::Prod => get_prod_config_file(),
        Env::Dev  => get_dev_config_file (),
    }
}


#[derive(Debug, Clone)]
pub struct Config {
    pub env:             Env,

    /// Exclude any folders with these names from the index scan
    pub folder_excludes: Vec<String>,

    /// Path to vault on disk (`~` as an alias for `$HOME`)
    pub vault_path:      PathBuf,

    // TODO: obsidian might just use the folder name? in which case, this is redundant
    /// Name of the vault as obsidian recognized it
    pub vault_name:      String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct YamlConfig {
    pub folder_excludes: Vec<String>,

    pub vault_folder:    PathBuf,

    pub vault_name:      String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            env:             Env::Dev,
            folder_excludes: Default::default(),
            vault_path:      Default::default(),
            vault_name:      Default::default(),
        }
    }
}

impl Config {
    pub fn with_excludes(excludes: Vec<String>) -> Self {
        let mut x = Self::default();
        x.folder_excludes = excludes;

        x
    }
}


fn get_prod_config_file() -> PathBuf {
    let conf = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)

        .unwrap_or_else(|_| env::home_dir()
            .expect("Unable to get home dir")
            .join  (".config/")
        )
    ;

    conf.join("copperminds.yaml")
}

fn get_dev_config_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config.dev.yaml")
}
