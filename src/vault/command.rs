use std::fmt::Debug;

use file_id::FileId;
use tokio::sync::{mpsc, oneshot};

use crate::vault::{VaultStats, fm::{FmAction, FmProperty, FmStatus}, md_file::{FileView, MdFile}};

// https://tokio.rs/tokio/tutorial/channels
#[derive(Debug)]
pub enum VaultCommand {
    IterFilesWith   (IterFilesWith,    Responder<Vec<FileView>>),
    OpenInObsidian  (OpenInObsidian,   Responder<()>),
    Register        (Register,         Responder<Subscriber<VaultUpdate>>),
    GetVaultStats   (GetVaultStats,    Responder<VaultStats>),
    NukeActionables (NukeActionables,  Responder<()>),

    ModifyFile      (ModifyFile,       Responder<Result<(), String>>),
    DeleteFile      (DeleteFile,       Responder<()>),
}



pub type Responder <T> = oneshot::Sender<T>;
pub type Subscriber<T> = mpsc   ::Receiver<T>;

// pub type Predicate = Box<dyn Fn(&MdFile) -> bool + Send>;
pub type Predicate = fn(&MdFile) -> bool;


pub struct IterFilesWith {
    pub filter: Predicate,
}

#[derive(Debug)]
pub struct ModifyFile {
    pub id:      FileId,
    pub changes: Vec<ModifyFileKind>,
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

#[derive(Debug)]
pub struct GetVaultStats {}

#[derive(Debug)]
pub struct NukeActionables {}



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

to_command!(IterFilesWith,    Vec<FileView>);
to_command!(OpenInObsidian,   ());
to_command!(Register,         Subscriber<VaultUpdate>);
to_command!(GetVaultStats,    VaultStats);
to_command!(ModifyFile,       Result<(), String>);
to_command!(DeleteFile,       ());
to_command!(NukeActionables,  ());

impl<T> Cmd<T> for VaultCommand {
    fn to_command(self, _: Responder<T>) -> VaultCommand {
        self
    }
}


#[derive(Debug, Clone, Copy)]
pub enum VaultUpdate {
    Rescan,
}

#[derive(Debug, Clone, Copy)]
pub enum ModifyFileKind {
    SetTypeInfo,
    SetAction(FmAction),
    SetStatus(FmStatus),
}


impl Debug for IterFilesWith {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IterFilesWith").field("filter", &self.filter).finish()
    }
}
