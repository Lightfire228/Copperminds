use std::{collections::HashMap, fmt::Display, sync::LazyLock};

use enum_iterator::{Sequence, all};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmProperty {
    Status,
    Type,
    Action,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// TODO: would probably be better to use a HashMap<>,
// since then you get "iter all values" for free
impl_get_key!(FmProperty,
    Status   => "status",
    Type     => "type",
    Action   => "action",
    Project  => "project",
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
