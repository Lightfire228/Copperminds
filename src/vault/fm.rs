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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmType {
    Info,
    Action,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmAction {
    Todo,
    Backlog,
    Entertainment,
    MaybeSomeday,
    WaitingFor,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
