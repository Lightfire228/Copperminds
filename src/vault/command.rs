use file_id::FileId;
use tokio::sync::oneshot::{self, Sender};

use crate::vault::{fm::FmProperty, md_file::{FileView, MdFile}};

// https://tokio.rs/tokio/tutorial/channels
#[derive(Debug)]
pub enum VaultCommand {
    IterFilesWith (IterFilesWith,  Responder<Vec<FileView>>),
    SetProperty   (SetProperty,    Responder<()>),
    OpenInObsidian(OpenInObsidian, Responder<()>),
}


pub type Responder<T> = oneshot::Sender<T>;

// Cannot be a Box<dyn Fn> because those aren't Send
pub type Predicate = fn(&MdFile) -> bool;


#[derive(Debug)]
pub struct IterFilesWith {
    pub filter: Predicate,
}

#[derive(Debug)]
pub struct SetProperty {
    pub id:     FileId,
    pub prop:   FmProperty,
    pub value:  String,
}

#[derive(Debug)]
pub struct OpenInObsidian {
    pub id: FileId,
}



pub trait Cmd<T> {
    fn to_command(self, tx: Sender<T>) -> VaultCommand;
}


macro_rules! to_command {
    ($name:ident, $type:ty) => {
        impl Cmd<$type> for $name {
            fn to_command(self, tx: Sender<$type>) -> VaultCommand {
                VaultCommand::$name(self, tx)
            }
        }
    };
}

to_command!(IterFilesWith,  Vec<FileView>);
to_command!(SetProperty,    ());
to_command!(OpenInObsidian, ());

impl<T> Cmd<T> for VaultCommand {
    fn to_command(self, _: Sender<T>) -> VaultCommand {
        self
    }
}
