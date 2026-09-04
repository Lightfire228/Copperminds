#![allow(dead_code)]


#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FmProperty {
    Inbox,
    Category,
    Status,
    Type,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmType {
    Info,
    Action,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmAction {
    Todo,
    Backlog,
    MaybeSomeday,
    WaitingFor,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmStatus {
    Completed,
    Complete,
    Archived,
    Archive,
}

impl FmStatus {
    pub fn _is_completed(&self) -> bool {
        match self {
            FmStatus::Completed => true,
            FmStatus::Complete  => true,
            FmStatus::Archived  => false,
            FmStatus::Archive   => false,
        }
    }

    pub fn _is_archived(&self) -> bool {
        match self {
            FmStatus::Completed => false,
            FmStatus::Complete  => false,
            FmStatus::Archived  => true,
            FmStatus::Archive   => true,
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
    Inbox    => "inbox",
    Category => "category",
    Status   => "status",
    Type     => "type",
    Action   => "action",
);

impl_get_key!(FmType,
    Info   => "info",
    Action => "action",
);

impl_get_key!(FmAction,
    WaitingFor   => "waiting_for",
    Todo         => "todo",
    Backlog      => "backlog",
    MaybeSomeday => "maybe_someday",
);

impl_get_key!(FmStatus,
    Completed => "completed",
    Complete  => "complete",
    Archived  => "archived",
    Archive   => "archive",
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
