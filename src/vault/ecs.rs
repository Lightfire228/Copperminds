mod components;

use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};

use file_id::FileId;
use log::warn;
use yaml_serde::Mapping;

use crate::{file_shit, vault::{file_utilities::RawFile, fm::{FmAction, FmProperty, FmStatus, FmType}, md_file::MdFile}};

#[derive(Default)]
pub struct ECS {

    file:    HashMap<FileId, File>,
    fm:      HashMap<FileId, FmComponent>,
    md_text: HashMap<FileId, MdTextComponent>,

    /// `/\s*/`
    empty:   HashSet<FileId>,

    // fm properties
    info:    HashSet<FileId>,
    action:  HashMap<FileId, ActionComponent>,
    status:  HashMap<FileId, StatusComponent>,

}

pub struct File {
    pub name:    String,
    pub unnamed: bool,

    /// is the absolute file path to the file
    pub path:    PathBuf,

    /// this is for debugging purposes
    /// mainly to confirm there haven't been any untracked changes since the last vault scan
    pub og_text: String,
}

pub struct FmComponent {
    pub fm: Mapping,
}

pub struct MdTextComponent {
    pub text: String,
}

pub struct TypeComponent {
    pub type_: FmType,
}

pub struct ActionComponent {
    pub action: FmAction,
}

pub struct StatusComponent {
    pub status: FmStatus,
}

pub struct NewFile {
    pub id:      FileId,

    pub path:    PathBuf,
    pub og_text: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Frontmatter,
    MdText,
    Empty,
    Info,
    Action,
    Status,
}

pub enum ComponentValue<'a> {
    Frontmatter(FileId, &'a FmComponent),
    MdText     (FileId, &'a MdTextComponent),
    Empty      (FileId),
    Info       (FileId),
    Action     (FileId, &'a ActionComponent),
    Status     (FileId, &'a StatusComponent),
}

pub enum ComponentValueIter<'a> {
    Frontmatter(Box<dyn Iterator<Item = (FileId, &'a FmComponent     )> + 'a>),
    MdText     (Box<dyn Iterator<Item = (FileId, &'a MdTextComponent )> + 'a>),
    Empty      (Box<dyn Iterator<Item = (FileId, ()                  )> + 'a>),
    Info       (Box<dyn Iterator<Item = (FileId, ()                  )> + 'a>),
    Action     (Box<dyn Iterator<Item = (FileId, &'a ActionComponent )> + 'a>),
    Status     (Box<dyn Iterator<Item = (FileId, &'a StatusComponent )> + 'a>),
}

impl ECS {
    #[allow(unused)]
    pub fn new() -> Self {
        Default::default()
    }

    #[allow(unused)]
    pub fn new_file(&mut self, file: NewFile) {

        // TODO: not this
        let moqup = MdFile {
            id:        FileId::Inode { device_id: 0, inode_number: 0 },
            path:      PathBuf::new(),
            raw_file:  RawFile::new(file.og_text.clone()),
            file_name: "".to_owned(),
        };

        self.file.insert(file.id, File {
            name:    file_shit::get_file_name(&file.path),
            og_text: file_shit::get_file_text(&file.path),
            unnamed: moqup.is_unnamed(),
            path:    file.path,
        });

        // TODO: get rid of the type property
        // it can be inferred from the other top level properties
        // - info
        //   - TODO: move the info from 'type' to a dedicated prop
        // - action


        self.add_info  (&moqup, file.id);
        self.add_status(&moqup, file.id);
        self.add_action(&moqup, file.id);
        self.add_empty (&moqup, file.id);

        let fm = moqup.raw_file.frontmatter;
        let md = moqup.raw_file.md_text;

        self.add_fm(fm, file.id);
        self.add_md(md, file.id);
    }

    fn add_info(&mut self, moqup: &MdFile, id: FileId) {
        if !moqup.is_type_info() {
            return;
        }

        self.info.insert(id);
    }

    fn add_status(&mut self, moqup: &MdFile, id: FileId) {
        let Some(status) = moqup.get_property(FmProperty::Status) else {
            return;
        };

        let Ok  (status) = status.as_str().try_into() else {
            warn!("unknown status prop: {status}");
            return;
        };

        self.status.insert(id, StatusComponent { status });
    }

    fn add_action(&mut self, moqup: &MdFile, id: FileId) {

        if !moqup.is_type_action() {
            return;
        }

        let Some(action) = moqup.get_property(FmProperty::Action) else {
            return;
        };

        let Ok  (action) = action.as_str().try_into() else {
            warn!("unknown action prop: {action}");
            return;
        };

        self.action.insert(id, ActionComponent { action });
    }

    fn add_empty(&mut self, moqup: &MdFile, id: FileId) {
        if !moqup.is_empty_raw() {
            return;
        }
        self.empty.insert(id);
    }

    fn add_fm(&mut self, fm: Option<Mapping>, id: FileId) {
        let Some(fm) = fm else {
            return;
        };

        self.fm.insert(id, FmComponent { fm });
    }

    fn add_md(&mut self, md: String, id: FileId) {
        self.md_text.insert(id, MdTextComponent { text: md });
    }


    pub fn get_by_component<'a>(&'a self, comp: ComponentKind) -> ComponentValueIter<'a> {

        type Kind   = ComponentKind;
        type Ci<'b> = ComponentValueIter<'b>;
        match comp {
            Kind::Frontmatter => Ci::Frontmatter(Box::new(self.fm     .iter().map(|x| (*x.0, x.1) ))),
            Kind::MdText      => Ci::MdText     (Box::new(self.md_text.iter().map(|x| (*x.0, x.1) ))),
            Kind::Empty       => Ci::Empty      (Box::new(self.empty  .iter().map(|x| (*x,   () ) ))),
            Kind::Info        => Ci::Info       (Box::new(self.info   .iter().map(|x| (*x,   () ) ))),
            Kind::Action      => Ci::Action     (Box::new(self.action .iter().map(|x| (*x.0, x.1) ))),
            Kind::Status      => Ci::Status     (Box::new(self.status .iter().map(|x| (*x.0, x.1) ))),
        }
    }

    pub fn get_has_all_components<'a>(&'a self, comp: &[ComponentKind]) -> Vec<FileId> {

        self
            .file
            .iter  ()
            .filter_map(|f| {
                comp
                    .iter()
                    .map (|c| self.get_component(*f.0, *c))
                    .all (|f| f.is_some())
                    .then_some(*f.0)
            })
            .collect()
    }

    pub fn get_component(&self, id: FileId, comp: ComponentKind) -> Option<ComponentValue> {
        match comp {
            ComponentKind::Frontmatter => self.fm     .get(&id).map(|x| (id, x).into()),
            ComponentKind::MdText      => self.md_text.get(&id).map(|x| (id, x).into()),
            ComponentKind::Empty       => self.empty  .get(&id).map(|_| ComponentValue::Empty(id)),
            ComponentKind::Info        => self.info   .get(&id).map(|_| ComponentValue::Info (id)),
            ComponentKind::Action      => self.action .get(&id).map(|x| (id, x).into()),
            ComponentKind::Status      => self.status .get(&id).map(|x| (id, x).into()),
        }
    }


    // fn query_component()
}


macro_rules! impl_into {
    ($( ($ident:ident, $comp:ty)),+ $(,)?) => {$(
        impl<'a> From<(FileId, &'a $comp)> for ComponentValue<'a> {
            fn from((id, value): (FileId, &'a $comp)) -> ComponentValue<'a> {
                ComponentValue::$ident(id, value)
            }
        })*
    };
}


impl_into!(
    (Frontmatter, FmComponent),
    (MdText,      MdTextComponent),
    (Action,      ActionComponent),
    (Status,      StatusComponent),
);




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecs_to_string() {
        todo!()
    }

}
