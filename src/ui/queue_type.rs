use std::fmt::Display;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    Inbox,
    Actionables,
}

impl Display for QueueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueType::Inbox       => write!(f, "Inbox"),
            QueueType::Actionables => write!(f, "Actionables"),
        }
    }
}
