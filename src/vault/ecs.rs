mod components;

use std::{collections::{HashMap, HashSet}, path::PathBuf};

use file_id::FileId;
use yaml_serde::Mapping;

use crate::{file_shit, vault::fm::{FmAction, FmStatus}};

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
    pub name: String,

    /// is the absolute file path to the file
    pub path: PathBuf,

    /// this is for debugging purposes
    /// mainly to confirm there haven't been any untracked changes since the last vault scan
    pub og_text: String,
}

pub struct FmComponent {
    pub fm: Mapping,
}

pub struct MdTextComponent {
    pub fm: String,
}

pub struct ActionComponent {
    pub action: FmAction,
}

pub struct StatusComponent {
    pub action: FmStatus,
}

pub struct NewFile {
    pub id:      FileId,

    pub name:    String,
    pub path:    PathBuf,
    pub og_text: String,

    // TODO: derive properties from this
    pub fm: Option<Mapping>,
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

type Iter<T> = dyn Iterator<Item = (FileId, T)>;

pub enum ComponentValueIter<'a> {
    Frontmatter(Box<dyn Iterator<Item = (FileId, &'a FmComponent     )> + 'a>),
    MdText     (Box<dyn Iterator<Item = (FileId, &'a MdTextComponent )> + 'a>),
    Empty      (Box<dyn Iterator<Item = (FileId, ()                  )> + 'a>),
    Info       (Box<dyn Iterator<Item = (FileId, ()                  )> + 'a>),
    Action     (Box<dyn Iterator<Item = (FileId, &'a ActionComponent )> + 'a>),
    Status     (Box<dyn Iterator<Item = (FileId, &'a StatusComponent )> + 'a>),
}

impl ECS {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_file(&mut self, file: NewFile) {
        self.file.insert(file.id, File {
            name:    file_shit::get_file_name(&file.path),
            og_text: file_shit::get_file_text(&file.path),
            path:    file.path,
        });

        // TODO: parse out components
        // - info
        // - actionable
        // - etc
    }

    pub fn get_by_component<'a>(&'a self, comp: ComponentKind) -> ComponentValueIter<'a> {

        match comp {
            ComponentKind::Frontmatter => ComponentValueIter::Frontmatter(Box::new(self.fm     .iter().map(|x| (*x.0, x.1)))),
            ComponentKind::MdText      => ComponentValueIter::MdText     (Box::new(self.md_text.iter().map(|x| (*x.0, x.1)))),
            ComponentKind::Empty       => ComponentValueIter::Empty      (Box::new(self.empty  .iter().map(|x| (*x,   () )))),
            ComponentKind::Info        => ComponentValueIter::Info       (Box::new(self.info   .iter().map(|x| (*x,   () )))),
            ComponentKind::Action      => ComponentValueIter::Action     (Box::new(self.action .iter().map(|x| (*x.0, x.1)))),
            ComponentKind::Status      => ComponentValueIter::Status     (Box::new(self.status .iter().map(|x| (*x.0, x.1)))),
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
