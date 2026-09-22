use yaml_serde::Mapping;

use crate::{config::Config, vault::{ecs::{File, FileId}, fm::*}};

pub trait Component: Send + 'static {}

impl<T> Component for T
where
    T: Send + 'static
{}


#[derive(Debug)]
pub struct FmComponent {
    pub fm: Mapping,
}

#[derive(Debug)]
pub struct MdTextComponent {
    pub text: String,
}

#[derive(Debug)]
pub struct TypeComponent {
    pub type_: FmType,
}

#[derive(Debug)]
pub struct ActionComponent {
    pub action: FmAction,
}

#[derive(Debug)]
pub struct StatusComponent {
    pub status: FmStatus,
}

#[derive(Debug)]
pub struct ProjectComponent {
    pub project: String,
}

#[derive(Debug)]
pub struct EmptyComponent;

#[derive(Debug)]
pub struct InfoComponent;


#[derive(Debug)]
pub struct EcsFileView<'a> {
    pub id:       FileId,
    pub file:     &'a File,
    pub fm:       Option<&'a FmComponent>,
    pub md_text:  Option<&'a MdTextComponent>,
    pub empty:    Option<&'a EmptyComponent>,
    pub type_:    Option<&'a TypeComponent>,
    pub info:     Option<&'a InfoComponent>,
    pub action:   Option<&'a ActionComponent>,
    pub status:   Option<&'a StatusComponent>,
    pub project:  Option<&'a ProjectComponent>,
}
