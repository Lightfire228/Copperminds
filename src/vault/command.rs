use file_id::FileId;
use tokio::sync::{mpsc, oneshot};

use crate::vault::{fm::FmProperty, md_file::{FileView, MdFile}};

// https://tokio.rs/tokio/tutorial/channels
#[derive(Debug)]
pub enum VaultCommand {
    IterFilesWith (IterFilesWith,  Responder<Vec<FileView>>),
    OpenInObsidian(OpenInObsidian, Responder<()>),
    Register      (Register,       Responder<Subscriber<VaultUpdate>>),

    SetProperty   (SetProperty,    Responder<()>),
    DeleteFile    (DeleteFile,     Responder<()>),
}

#[derive(Debug)]
pub enum VaultCommandOpts {
    IterFilesWith (IterFilesWith),
    OpenInObsidian(OpenInObsidian),
    Register      (Register),

    SetProperty   (SetProperty),
    DeleteFile    (DeleteFile),
}



pub type Responder <T> = oneshot::Sender<T>;
pub type Subscriber<T> = mpsc   ::Receiver<T>;

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
pub struct DeleteFile {
    pub id: FileId,
}

#[derive(Debug)]
pub struct OpenInObsidian {
    pub id: FileId,
}

#[derive(Debug)]
pub struct Register {}



pub trait Cmd<T> {
    fn to_command(self, tx: Responder<T>) -> VaultCommand;
}


macro_rules! to_command {
    ($name:ident, $type:ty) => {
        impl Cmd<$type> for $name {
            fn to_command(self, tx: Responder<$type>) -> VaultCommand {
                VaultCommand::$name(self, tx)
            }
        }
    };
}

to_command!(IterFilesWith,  Vec<FileView>);
to_command!(OpenInObsidian, ());
to_command!(Register,       Subscriber<VaultUpdate>);
to_command!(SetProperty,    ());
to_command!(DeleteFile,     ());

impl<T> Cmd<T> for VaultCommand {
    fn to_command(self, _: Responder<T>) -> VaultCommand {
        self
    }
}


#[derive(Debug, Clone, Copy)]
pub enum VaultUpdate {
    Rescan,
}
