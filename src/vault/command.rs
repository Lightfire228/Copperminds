use tokio::sync::oneshot;

use crate::vault::{fm::FmProperty, md_file::{FileId, FileView, MdFile}};

// https://tokio.rs/tokio/tutorial/channels
#[derive(Debug)]
pub enum VaultCommand {
    IterFilesWith {
        filter: Predicate,
        resp:   Responder<Vec<FileView>>,
    },
    SetProperty {
        id:     FileId,
        prop:   FmProperty,
        value:  String,
        resp:   Responder<()>,
    },
}


pub type Responder<T> = oneshot::Sender<T>;

// Cannot be a Box<dyn Fn> because those aren't Send
pub type Predicate = fn(&MdFile) -> bool;
