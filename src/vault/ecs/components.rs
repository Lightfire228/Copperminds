use yaml_serde::Mapping;

use crate::{config::Config, vault::{ecs::{File, FileId}, fm::*}};

pub trait Component: Send + 'static {}

impl<T> Component for T
where
    T: Send + 'static
{}

#[derive(Debug, Default)]
pub struct FmComponent {
    pub fm:       Mapping,
    pub type_:    FmProp<FmType>,
    pub action:   FmProp<FmAction>,
    pub status:   FmProp<FmStatus>,
    // pub project:  FmProp<FmProject>,
    pub modified: bool,
}


#[derive(Debug)]
pub struct FmProp<T> {
    pub value:    Option<T>,
    pub modified: bool,
}

#[derive(Debug)]
pub struct MdTextComponent {
    pub text: String,
}

/// Tracks if the file, before parsing frontmatter, has any non-whitespace character in it
#[derive(Debug)]
pub struct EmptyFileComponent;

/// Tracks if the file has a name that's illegal on Android
#[derive(Debug)]
pub struct IllegalNameComponent;



impl<T> Default for FmProp<T> {
    fn default() -> Self {
        Self {
            value:    None,
            modified: false,
        }
    }
}
