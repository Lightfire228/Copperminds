pub mod parsing;

use std::{collections::HashMap, fmt::Display, sync::LazyLock};

use enum_iterator::{Sequence, all};
use yaml_serde::{Mapping, Value, to_value};

use crate::vault::ecs::components::FmProp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmProperty {
    Status,
    Type,
    Action,
    Project,
    Tags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum FmType {
    Info,
    Action,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum FmAction {
    Todo,
    Backlog,
    Entertainment,
    MaybeSomeday,
    WaitingFor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum FmStatus {
    Completed,
    Archived,
}


#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub struct FmTags {
    pub tags: Vec<String>,
}



#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FmProject {
    pub name: String,
    pub is_workspace: bool,
}



impl FmStatus {
    pub fn _is_completed(&self) -> bool {
        match self {
            FmStatus::Completed => true,
            FmStatus::Archived  => false,
        }
    }

    pub fn _is_archived(&self) -> bool {
        match self {
            FmStatus::Completed => false,
            FmStatus::Archived  => true,
        }
    }
}

// impl From<Mapping> for FrontmatterYaml {
//     fn from(value: Mapping) -> Self {
//         Self {
//             type_:   FmType::try_from(value),
//             action:  todo!(),
//             status:  todo!(),
//             project: todo!(),
//         }
//     }
// }

pub trait GetKey {
    fn get_key(&self) -> String;
}

macro_rules! impl_get_key {
    ($type:ident, $($variant:ident => $value:literal,)+ ) => {
        impl GetKey for $type {
            fn get_key(&self) -> String {
                match &self {
                    $(
                        $type::$variant => $value.to_owned(),
                    )+
                }
            }
        }
    };
}

impl_get_key!(FmProperty,
    Status    => "status",
    Type      => "type",
    Action    => "action",
    Project   => "project",
    Tags      => "tag",
);

impl_get_key!(FmType,
    Info   => "info",
    Action => "action",
);

impl_get_key!(FmAction,
    WaitingFor    => "waiting_for",
    Todo          => "todo",
    Backlog       => "backlog",
    Entertainment => "entertainment",
    MaybeSomeday  => "maybe_someday",
);

impl_get_key!(FmStatus,
    Completed => "completed",
    Archived  => "archived",
);

impl GetKey for String {
    fn get_key(&self) -> String {
        self.clone()
    }
}

impl GetKey for &str {
    fn get_key(&self) -> String {
        self.to_string()
    }
}


impl Display for FmAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FmAction::Todo          => write!(f, "Todo"),
            FmAction::Backlog       => write!(f, "Backlog"),
            FmAction::Entertainment => write!(f, "Entertainment"),
            FmAction::MaybeSomeday  => write!(f, "Maybe Someday"),
            FmAction::WaitingFor    => write!(f, "Waiting For"),
        }
    }
}

impl Display for FmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FmStatus::Completed => write!(f, "Completed"),
            FmStatus::Archived  => write!(f, "Archived"),
        }
    }
}

impl TryFrom<&str> for FmType {
    type Error = ();

    fn try_from(value: &str) -> Result<FmType, Self::Error> {
        Ok(match value {
            "action" => FmType::Action,
            "info"   => FmType::Info,
            _        => Err(())?
        })
    }
}

macro_rules! try_from_get_key {
    ($name:ident) => {

        impl TryFrom<&str> for $name {
            type Error = ();

            fn try_from(value: &str) -> Result<$name, Self::Error> {

                static TABLE: LazyLock<HashMap<String, $name>> = LazyLock::new(|| $name::all().into_iter().map(|x| (x.get_key(), x)).collect());

                TABLE.get(value).cloned().ok_or(())
            }
        }

    };
}

try_from_get_key!(FmAction);

impl TryFrom<&str> for FmStatus {
    type Error = ();

    fn try_from(value: &str) -> Result<FmStatus, Self::Error> {

        Ok(match value {
            "complete"  |
            "completed" => FmStatus::Completed,
            "archive"   |
            "archived"  => FmStatus::Archived,
            _           => Err(())?
        })
    }
}


impl FmAction {
    fn all() -> Vec<Self> {
        all::<FmAction>().collect()
    }
}


macro_rules! impl_value_from_str {
    ($ident:ident) => {

        impl From<$ident> for Value {
            fn from(value: $ident) -> Self {
                Value::String(value.get_key())
            }
        }

    };
}

impl_value_from_str!(FmProperty);
impl_value_from_str!(FmType);
impl_value_from_str!(FmAction);
impl_value_from_str!(FmStatus);


impl From<FmTags> for Value {
    fn from(value: FmTags) -> Self {
        let sequence: Vec<_> = value.tags.iter().map(|x| Value::String(x.clone())).collect();
        Value::Sequence(sequence.into())
    }
}



impl FmAction {
    // TODO: move this to a parse rule table or something
    pub fn from_alias(value: &str) -> Option<Self> {
        Some(match value {
            "t" => FmAction::Todo,
            "b" => FmAction::Backlog,
            "e" => FmAction::Entertainment,
            "m" => FmAction::MaybeSomeday,
            "w" => FmAction::WaitingFor,
            _ => None?
        })
    }
}


impl FmStatus {
    // TODO: move this to a parse rule table or something
    pub fn from_alias(value: &str) -> Option<Self> {
        Some(match value {
            "c" => FmStatus::Completed,
            "a" => FmStatus::Archived,
            _ => None?
        })
    }
}




pub fn update_prop_str<'a, T: GetKey>(fm: &mut Mapping, prop: FmProperty, value: Option<&T>) {
    let key = Value::String(prop.get_key());

    let Some(value) = value else {
        fm.remove(key);
        return;
    };

    let value = Value::String(value.get_key());

    let slot = fm.entry(key).or_insert_with(|| Value::Null);

    *slot = value;
}



pub fn update_prop_proj<'a>(fm: &mut Mapping, prop: FmProperty, value: Option<&FmProject>) {
    let key = Value::String(prop.get_key());

    let Some(value) = value else {
        fm.remove(key);
        return;
    };


    let value = if value.is_workspace {
        to_value(value).expect("Unable to serialize FmProject")
    }
    else {
        Value::String(value.name.clone())
    };

    let slot = fm.entry(key).or_insert_with(|| Value::Null);

    *slot = value;
}

pub fn update_prop_list<'a>(fm: &mut Mapping, prop: FmProperty, values: Option<&[String]>) {

    let key = Value::String(prop.get_key());

    let Some(values) = values else {
        fm.remove(key);
        return;
    };

    let value: Value = to_value(values).expect("unable to serialize list: {prop:?}");

    let slot = fm.entry(key).or_insert_with(|| Value::Null);

    *slot = value;
}
