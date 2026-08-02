

#[derive(Debug, Clone)]
pub enum FmProperty {
    Inbox,
    Category,
    Status,
    Type,
    Action,
}

#[derive(Debug, Clone)]
pub enum FmType {
    Info,
    Action,
}
#[derive(Debug, Clone)]
pub enum FmAction {
    WaitingFor,
    Calendar,
    Todo,
    MaybeSomeday,
    Project,
}
#[derive(Debug, Clone)]
pub enum FmStatus {
    Completed,
    Complete,
    Archived,
    Archive,
}

impl FmStatus {
    pub fn is_completed(&self) -> bool {
        match self {
            FmStatus::Completed => true,
            FmStatus::Complete  => true,
            _ => false,
        }
    }

    pub fn is_archived(&self) -> bool {
        match self {
            FmStatus::Archived => true,
            FmStatus::Archive  => true,
            _ => false,
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
    Calendar     => "calendar",
    Todo         => "todo",
    MaybeSomeday => "maybe_someday",
    Project      => "maybe_someday",
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
        (*self).to_owned()
    }
}
