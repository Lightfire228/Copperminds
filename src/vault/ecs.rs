mod delete_empty_unnamed;
mod parse;
mod mut_props;
mod to_text;

use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};

use enum_iterator::{Sequence, all};
use log::{debug, warn};
use yaml_serde::Mapping;


type FileId = file_id::FileId;

use crate::vault::fm::{FmAction, FmStatus, FmType};

use super::build_regex;

#[derive(Default)]
pub struct Ecs {

    file:    HashMap<FileId, File>,
    fm:      HashMap<FileId, FmComponent>,
    md_text: HashMap<FileId, MdTextComponent>,

    /// `/\s*/`
    empty:   HashSet<FileId>,

    // fm properties
    type_:   HashMap<FileId, TypeComponent>,
    info:    HashSet<FileId>,
    action:  HashMap<FileId, ActionComponent>,
    status:  HashMap<FileId, StatusComponent>,

}

#[derive(Debug)]
pub struct File {
    pub name:     String,
    pub unnamed:  bool,

    /// is the absolute file path to the file
    pub path:     PathBuf,

    /// this is for debugging purposes
    /// mainly to confirm there haven't been any untracked changes since the last vault scan
    pub raw_text: String,
}

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


pub struct NewFile {
    pub id:       FileId,

    pub path:     PathBuf,
    pub raw_text: String,
    pub name:     String,
}

#[derive(Debug)]
pub struct FileView<'a> {
    pub id:       FileId,
    pub file:     &'a File,
    pub fm:       Option<&'a FmComponent>,
    pub md_text:  Option<&'a MdTextComponent>,
    pub is_empty: bool,
    pub type_:    Option<&'a TypeComponent>,
    pub info:     bool,
    pub action:   Option<&'a ActionComponent>,
    pub status:   Option<&'a StatusComponent>,
}

#[derive(Debug, Clone, Copy, Sequence)]
pub enum ComponentKind {
    Frontmatter,
    MdText,
    Empty,
    Type,
    Info,
    Action,
    Status,
}

pub enum ComponentQuery<'a> {
    Frontmatter(Option<&'a FmComponent>),
    MdText     (Option<&'a MdTextComponent>),
    Empty      (bool),
    Type       (Option<&'a TypeComponent>),
    Info       (bool),
    Action     (Option<&'a ActionComponent>),
    Status     (Option<&'a StatusComponent>),
}

pub enum ComponentQueryIter<'a> {
    Frontmatter(Box<dyn Iterator<Item = (&'a FileId, &'a FmComponent    )> + 'a>),
    MdText     (Box<dyn Iterator<Item = (&'a FileId, &'a MdTextComponent)> + 'a>),
    Empty      (Box<dyn Iterator<Item =  &'a FileId                      > + 'a>),
    Type       (Box<dyn Iterator<Item = (&'a FileId, &'a TypeComponent  )> + 'a>),
    Info       (Box<dyn Iterator<Item =  &'a FileId                      > + 'a>),
    Action     (Box<dyn Iterator<Item = (&'a FileId, &'a ActionComponent)> + 'a>),
    Status     (Box<dyn Iterator<Item = (&'a FileId, &'a StatusComponent)> + 'a>),
}

impl Ecs {
    pub fn new() -> Self {
        Default::default()
    }

    // --- queries

    pub fn get_all(&self) -> impl Iterator<Item = FileView<'_>> {
        self.file.iter().map(|x| self.to_file_view(*x.0, x.1))
    }

    pub fn get(&self, id: FileId) -> Option<FileView<'_>> {
        self.file.get(&id).map(|x| self.to_file_view(id, x))
    }

    fn to_file_view<'a>(&'a self, id: FileId, file: &'a File) -> FileView<'a> {
        FileView {
            id,
            file,
            fm:       self.get_fm_component    (id),
            md_text:  self.get_md_component    (id),
            is_empty: self.is_empty            (id),
            type_:    self.get_type_component  (id),
            info:     self.has_info_component  (id),
            action:   self.get_action_component(id),
            status:   self.get_status_component(id),
        }
    }

    pub fn get_component_counts<'a>(&'a self, comp: ComponentKind) -> usize {

        type Kind   = ComponentKind;
        match comp {
            Kind::Frontmatter => self.fm     .len(),
            Kind::MdText      => self.md_text.len(),
            Kind::Empty       => self.empty  .len(),
            Kind::Type        => self.type_  .len(),
            Kind::Info        => self.info   .len(),
            Kind::Action      => self.action .len(),
            Kind::Status      => self.status .len(),
        }
    }


    pub fn get_fm_component(&self, id: FileId) -> Option<&FmComponent> {
        self.fm.get(&id)
    }
    pub fn get_md_component(&self, id: FileId) -> Option<&MdTextComponent> {
        self.md_text.get(&id)
    }
    pub fn is_empty(&self, id: FileId) -> bool {
        self.empty.get(&id).is_some()
    }
    pub fn get_type_component(&self, id: FileId) -> Option<&TypeComponent> {
        self.type_.get(&id)
    }
    pub fn has_info_component(&self, id: FileId) -> bool {
        self.info.get(&id).is_some()
    }
    pub fn get_action_component(&self, id: FileId) -> Option<&ActionComponent> {
        self.action.get(&id)
    }
    pub fn get_status_component(&self, id: FileId) -> Option<&StatusComponent> {
        self.status.get(&id)
    }

    pub fn query_component(&self, id: FileId, comp: ComponentKind) -> ComponentQuery<'_> {
        match comp {
            ComponentKind::Frontmatter => ComponentQuery::Frontmatter(self.get_fm_component    (id)),
            ComponentKind::MdText      => ComponentQuery::MdText     (self.get_md_component    (id)),
            ComponentKind::Empty       => ComponentQuery::Empty      (self.is_empty            (id)),
            ComponentKind::Type        => ComponentQuery::Type       (self.get_type_component  (id)),
            ComponentKind::Info        => ComponentQuery::Info       (self.has_info_component  (id)),
            ComponentKind::Action      => ComponentQuery::Action     (self.get_action_component(id)),
            ComponentKind::Status      => ComponentQuery::Status     (self.get_status_component(id)),
        }
    }

    pub fn query_component_all(&self, comp: ComponentKind) -> ComponentQueryIter<'_> {
        match comp {
            ComponentKind::Frontmatter => ComponentQueryIter::Frontmatter(Box::new(self.fm     .iter())),
            ComponentKind::MdText      => ComponentQueryIter::MdText     (Box::new(self.md_text.iter())),
            ComponentKind::Empty       => ComponentQueryIter::Empty      (Box::new(self.empty  .iter())),
            ComponentKind::Type        => ComponentQueryIter::Type       (Box::new(self.type_  .iter())),
            ComponentKind::Info        => ComponentQueryIter::Info       (Box::new(self.info   .iter())),
            ComponentKind::Action      => ComponentQueryIter::Action     (Box::new(self.action .iter())),
            ComponentKind::Status      => ComponentQueryIter::Status     (Box::new(self.status .iter())),
        }
    }

    // --- writes

    /// Removes file from all components.
    /// NOTE: does not delete file from disk
    pub fn remove_file(&mut self, id: FileId) -> File {

        let components = ComponentKind::all();

        for cmp in components {
            match cmp {
                ComponentKind::Frontmatter => { self.fm     .remove(&id); },
                ComponentKind::MdText      => { self.md_text.remove(&id); },
                ComponentKind::Empty       => { self.empty  .remove(&id); },
                ComponentKind::Type        => { self.type_  .remove(&id); },
                ComponentKind::Info        => { self.info   .remove(&id); },
                ComponentKind::Action      => { self.action .remove(&id); },
                ComponentKind::Status      => { self.status .remove(&id); },
            }
        }

        self.file.remove(&id).unwrap()
    }
}



impl<'a> FileView<'a> {
    pub fn status_eq(&'a self, status: FmStatus) -> bool {
        self.status.is_some_and(|s| s.status == status)
    }
    pub fn action_eq(&'a self, action: FmAction) -> bool {
        self.action.is_some_and(|a| a.action == action)
    }
    pub fn type_eq(&'a self, type_: FmType) -> bool {
        self.type_.is_some_and(|t| t.type_ == type_)
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
               s.status != FmStatus::Archived
            && s.status != FmStatus::Completed
        })
    }

    pub fn needs_type(&'a self) -> bool {
        self.type_.is_none()
        && !self.file.path
            .ancestors()
            // TODO: move this to a config
            .any      (|p| p.ends_with("03 Data"))
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
        build_regex!(RE = r"(?i)^([\d \-_]*|Untitled(\s.*?)?)\.md$");

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

    pub fn to_file_text(&self) -> String {
        let md = self.get_md_text();

        let Some(FmComponent { fm }) = self.fm else {
            return md;
        };


        format!("---\n{}---\n{}", fm_to_text(&fm), md)
    }


    fn get_md_text(&self) -> String {
        return self
            .md_text
            .map(|x| x.text.to_string())
            .unwrap_or_default()
        ;
    }
}

fn fm_to_text(fm: &Mapping) -> String {
    yaml_serde::to_string(fm).unwrap()
}


impl ComponentKind {
    fn all() -> Vec<Self> {
        all::<ComponentKind>().collect()
    }
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};

    use yaml_serde::{Mapping, Value};

    use crate::{test_utils::{id, mapping_to_str}, vault::fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}};

    use super::*;

    macro_rules! fm {
        ($ecs:ident, $($key:expr => $value:expr),*$(,)? ) => {{
            #[allow(unused_mut)] // reason: ignore warning for empty fm creation
            let mut fm = Mapping::new();

            $(
                fm.insert(Value::String($key.get_key()), Value::String($value.get_key()));
            )*

            let id = id();

            $ecs.new_file(NewFile {
                id,
                path:     PathBuf::new(),
                raw_text: mapping_to_str(fm),
                name:     String::new(),
            });

            id
        }};
    }

    #[test]
    fn test_type_sorting() {

        let mut ecs = Ecs::new();

        let untyped = fm!(ecs, );
        let info    = fm!(ecs, FmProperty::Type => FmType::Info);
        let action  = fm!(ecs, FmProperty::Type => FmType::Action);


        let untyped = ecs.get(untyped).unwrap();
        let info    = ecs.get(info)   .unwrap();
        let action  = ecs.get(action) .unwrap();

        assert_eq!(untyped.needs_type(), true , "untyped");
        assert_eq!(info   .needs_type(), false, "info");
        assert_eq!(action .needs_type(), false, "action");

        assert!(info  .info);
        assert!(info  .type_eq(FmType::Info));
        assert!(action.type_eq(FmType::Action));
    }

    #[test]
    fn test_action_sorting() {

        let mut ecs = Ecs::new();

        let no_action_info     = fm!(ecs, FmProperty::Type => FmType::Info);
        let no_action          = fm!(ecs,                                     FmProperty::Action => FmAction::Todo);
        let needs_action       = fm!(ecs, FmProperty::Type => FmType::Action);
        let action_todo        = fm!(ecs, FmProperty::Type => FmType::Action, FmProperty::Action => FmAction::Todo);
        let action_waiting_for = fm!(ecs, FmProperty::Type => FmType::Action, FmProperty::Action => FmAction::WaitingFor);


        let no_action_info     = ecs.get(no_action_info)    .unwrap();
        let no_action          = ecs.get(no_action)         .unwrap();
        let needs_action       = ecs.get(needs_action)      .unwrap();
        let action_todo        = ecs.get(action_todo)       .unwrap();
        let action_waiting_for = ecs.get(action_waiting_for).unwrap();

        assert_eq!(no_action_info    .needs_action_assigned(), false, "no_action_info     needs_action_assigned");
        assert_eq!(no_action         .needs_action_assigned(), false, "no_action          needs_action_assigned");
        assert_eq!(needs_action      .needs_action_assigned(), true,  "needs_action       needs_action_assigned");
        assert_eq!(action_todo       .needs_action_assigned(), false, "action_todo        needs_action_assigned");
        assert_eq!(action_waiting_for.needs_action_assigned(), false, "action_waiting_for needs_action_assigned");

        assert_eq!(no_action_info    .is_actionable(),         false, "no_action_info     is_actionable");
        assert_eq!(no_action         .is_actionable(),         false, "no_action          is_actionable");
        assert_eq!(needs_action      .is_actionable(),         false, "needs_action       is_actionable");
        assert_eq!(action_todo       .is_actionable(),         true,  "action_todo        is_actionable");
        assert_eq!(action_waiting_for.is_actionable(),         true,  "action_waiting_for is_actionable");
    }

    #[test]
    fn test_status_sorting() {
        let mut ecs = Ecs::new();

        let archive   = fm!(ecs, FmProperty::Status => "archive");
        let archived  = fm!(ecs, FmProperty::Status => FmStatus::Archived);
        let complete  = fm!(ecs, FmProperty::Status => "complete");
        let completed = fm!(ecs, FmProperty::Status => FmStatus::Completed);

        let archive   = ecs.get(archive  ).unwrap();
        let archived  = ecs.get(archived ).unwrap();
        let complete  = ecs.get(complete ).unwrap();
        let completed = ecs.get(completed).unwrap();

        assert_eq!(archive  .is_archived(),  true,  "archive   is_archived");
        assert_eq!(archived .is_archived(),  true,  "archived  is_archived");
        assert_eq!(complete .is_archived(),  false, "complete  is_archived");
        assert_eq!(completed.is_archived(),  false, "completed is_archived");

        assert_eq!(archive  .is_completed(), false, "archive   is_completed");
        assert_eq!(archived .is_completed(), false, "archived  is_completed");
        assert_eq!(complete .is_completed(), true,  "complete  is_completed");
        assert_eq!(completed.is_completed(), true,  "completed is_completed");
    }
}
