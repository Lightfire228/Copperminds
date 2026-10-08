use yaml_serde::Mapping;

use crate::{prelude::build_regex, vault::{ecs::{File, FileId, components::*}, fm::{FmAction, FmStatus, FmType}}};


#[derive(Debug)]
pub struct EcsFileView<'a> {
    pub id:       FileId,
    pub file:     &'a File,
    pub fm:       Option<&'a FmComponent>,
    pub md_text:  Option<&'a MdTextComponent>,
    pub empty:    bool,
    pub type_:    Option<&'a FmType>,
    pub action:   Option<&'a FmAction>,
    pub status:   Option<&'a FmStatus>,
    // pub project:  Option<&'a FmProject>,
}



impl<'a> EcsFileView<'a> {
    pub fn status_eq(&'a self, status: FmStatus) -> bool {
        self.status == Some(&status)
    }
    pub fn action_eq(&'a self, action: FmAction) -> bool {
        self.action == Some(&action)
    }
    pub fn type_eq(&'a self, type_: FmType) -> bool {
        self.type_ == Some(&type_)
    }

    // TODO: remove the type prop and just infer the type from it's top level props
    // - info
    // - action
    /// Does not account for status prop
    pub fn is_actionable(&'a self) -> bool {
        self.type_eq(FmType::Action) && self.action.is_some()
    }

    pub fn is_open(&'a self) -> bool {
        self.status.is_none_or(|s| {
               s != &FmStatus::Archived
            && s != &FmStatus::Completed
        })
    }

    pub fn is_info(&'a self) -> bool {
        self.type_eq(FmType::Info)
    }

    pub fn is_empty(&'a self) -> bool {
        self.empty
    }

    pub fn needs_type(&'a self) -> bool {
        self.type_.is_none()
    }

    pub fn needs_sorting(&'a self) -> bool {
        [
            self.needs_type(),
            self.needs_action_assigned(),
            self.is_unnamed(),
        ]
            .iter()
            .any(|f| *f)
    }

    pub fn is_unnamed(&'a self) -> bool {
        // (?i) - sets case insensitivity
        build_regex!(RE = r"(?i)^([\d \-_]*|Untitled([\s\d\-_\(\)]*?)?)\.md$");

        RE.is_match(&self.file.name)
    }

    pub fn needs_action_assigned(&'a self) -> bool {
        self.type_eq(FmType::Action) && self.action.is_none()
    }

    pub fn is_archived(&'a self) -> bool {
        self.status_eq(FmStatus::Archived)
    }

    pub fn is_completed(&'a self) -> bool {
        self.status_eq(FmStatus::Completed)
    }

}
