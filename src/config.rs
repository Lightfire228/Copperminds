use std::{collections::HashMap, env, fs, path::{self, PathBuf}, time::Duration};
use serde::{Deserialize, Serialize};

mod yaml;

use crate::ui::filter::{UiFilter};

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


use crate::{backup, file_shit};

pub fn get_config(env: Env) -> Config {

    let file = get_config_file(env);

    let Ok(text) = fs::read_to_string(&file) else {
        panic!("unable to read config file: {file:?}")
    };

    yaml::parse_config_file(env, &text)
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

    /// derived from `vault_path`
    pub vault_name:      String,

    /// determines how long to wait before issuing another obsidian cli command
    pub open_timeout:    Duration,

    pub queues:          Vec<ConfigQueue>,
}

#[derive(Debug, Clone)]
pub struct ConfigQueue {
    pub name:   String,
    pub filter: UiFilter,
}


impl Default for Config {
    fn default() -> Self {
        Self {
            env:             Env::Dev,
            folder_excludes: Default::default(),
            vault_path:      Default::default(),
            vault_name:      Default::default(),
            open_timeout:    Default::default(),
            queues:          Default::default(),
        }
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
