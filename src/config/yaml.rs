use std::{collections::HashMap, path::PathBuf, time::Duration, fmt::Debug};
use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::{config::{Config, ConfigQueue, Env}, file_shit, ui::filter::{FilterByList::{self, All}, UiFilter}, vault::fm::FmAction};

pub fn parse_config_file(env: Env, text: &str) -> Config {
    let mut config: YamlConfig = yaml_serde::from_str(&text)
        .inspect_err(|err| panic!("unable to parse config file: {err}"))
        .unwrap()
    ;

    config.vault_folder = file_shit::resolve_tilde(config.vault_folder);

    if !config.vault_folder.exists() {
        panic!("vault folder does not exist");
    }

    let vault_name = file_shit::get_folder_name(&config.vault_folder);

    Config {
        env,
        folder_excludes: config.folder_excludes,
        vault_path:      config.vault_folder,
        vault_name:      vault_name,

        // TODO: read from file
        open_timeout:    Duration::from_millis(100),


        queues: config.queues.into_iter()
            .map(|(name, value)| Ok(ConfigQueue {
                name,
                filter: value.try_into()?
            }))
            .collect::<Result<Vec<_>, String>>()
            .unwrap()
        ,
    }
}


#[derive(Debug, Default, Serialize, Deserialize, Clone) ]
struct YamlConfig {
    pub folder_excludes: Vec<String>,
    pub vault_folder:    PathBuf,
    pub queues:          HashMap<String, YamlQueue>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone) ]
struct YamlQueue {
    pub needs_sorted: Option<bool>,
    pub is_open:      Option<bool>,
    pub by_action:    Option<YamlFilterList>,
    pub by_tags:      Option<YamlFilterList>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone) ]
struct YamlFilterList {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl TryFrom<YamlQueue> for UiFilter {
    type Error = String;

    fn try_from(val: YamlQueue) -> Result<Self, Self::Error> {

        Ok(Self {
            by_action:    val.by_action.try_into()?,
            by_tags:      val.by_tags  .try_into().expect("infallible"),
            needs_sorted: val.needs_sorted,
            is_open:      val.is_open,

            // TODO:
            by_type:      Default::default(),
            by_status:    Default::default(),
            by_project:   Default::default(),
        })
    }
}



impl<T, Err> TryFrom<Option<YamlFilterList>> for FilterByList<T>
where for <'a>
    T: TryFrom<&'a str, Error = Err>,
{
    type Error = Err;

    fn try_from(value: Option<YamlFilterList>) -> Result<Self, Self::Error> {
        value.map_or_else(
            ||  Ok(FilterByList::default()),
            |x| x.try_into()
        )
    }
}

impl<T, Err> TryFrom<YamlFilterList> for FilterByList<T>
where for <'a>
    T: TryFrom<&'a str, Error = Err>,
{

    type Error = Err;

    fn try_from<'a>(val: YamlFilterList) -> Result<Self, Err> {

        if let Some(list) = val.include {
            Ok(Self::Include(to_list(&list)?))
        }
        else if let Some(list) = val.exclude {
            Ok(Self::Exclude(to_list(&list)?))
        }
        else {
            Ok(Default::default())
        }
    }
}

fn to_list<T, Err>(list: &[String]) -> Result<Vec<T>, Err>
where
    for <'a> T: TryFrom<&'a str, Error = Err>,
{
    list
        .iter   ()
        .map    (|x| x.as_str().try_into())
        .collect()
}
