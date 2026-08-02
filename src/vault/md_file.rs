
use std::{fs, path::{Path, PathBuf}};

use crate::vault::{file_utilities::RawFile};

use super::regex;

pub type FileId = usize;

#[derive(Debug)]
pub struct MdFile {
    pub id:               FileId,
    pub path:             PathBuf,
    pub raw_file:         RawFile,

    /// Includes file extension
    pub file_name:        String,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum FmProperty {
    Inbox,
    Category,
    Status,
    Type,
    Action,
}

#[derive(Debug, Clone, Copy)]
pub enum _FmType {
    Info,
    Action,
}
#[derive(Debug, Clone, Copy)]
pub enum _FmAction {
    WaitingFor,
    Calendar,
    Todo,
    MaybeSomeday,
}
#[derive(Debug, Clone, Copy)]
pub enum _FmStatus {
    Completed,
    Archived,
}

impl MdFile {

    pub fn new(id: FileId, path: PathBuf) -> Self {
        let text = fs::read_to_string(&path).unwrap();

        Self {
            id,
            file_name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            path,
            raw_file:  RawFile::new(text),
        }
    }

    // --- queries

    pub fn is_empty(&self) -> bool {
        self.raw_file.is_empty()
    }

    pub fn _is_md_empty(&self) -> bool {
        self.raw_file.is_md_empty()
    }

    pub fn is_unnamed(&self) -> bool {
        regex!(RE = r"^([\d \-_]*|Untitled.*?)\.md$");

        RE.is_match(&self.file_name)
    }

    /// Does not check subdirectories
    pub fn _is_in_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>
    {
        self
            .path
            .parent()
            .is_some_and(|f|
                f.ends_with(path)
            )
    }

    /// GTD Type
    pub fn is_untyped(&self) -> bool {
        self.get_property(FmProperty::Type).is_none()
        && !self.path
            .ancestors()
            .any      (|p| p.ends_with("03 Data"))
    }

    /// GTD Action
    /// - waiting for
    /// - todo
    /// - maybe someday
    ///
    pub fn is_unactioned(&self) -> bool {
        self.is_actionable()
        && !self.get_property(FmProperty::Action).is_some()
    }

    pub fn is_actionable(&self) -> bool {
        self
            .get_property(FmProperty::Type)
            .is_some_and(|p| p == "action")
    }

    pub fn is_archived(&self) -> bool {
        self.is_property_any_of(FmProperty::Status, &["archive",  "archived"])
    }

    pub fn is_complete(&self) -> bool {
        self.is_property_any_of(FmProperty::Status, &["complete", "completed"])
    }

    pub fn get_property(&self, property: FmProperty) -> Option<String> {
        self
            .raw_file
            .get_property(&property.get_key())
            .ok()
    }

    pub fn is_property(&self, property: FmProperty, value: &str) -> bool {
        self.get_property(property).is_some_and(|p| p == value)
    }

    pub fn is_property_any_of(&self, property: FmProperty, vals: &[&str]) -> bool {
        self
            .get_property(property)
            .is_some_and(|p|
                vals.contains(&p.as_str())
            )
    }

    // ---- writes


    pub fn refresh(&mut self) {
        *self = Self::new(self.id, self.path.clone());
    }

    pub fn write_file(&self) {
        self.raw_file.write(&self.path);
    }


    pub fn set_property(&mut self, property: FmProperty, value: String) {
        self.raw_file.set_property(property.get_key(), value);
    }

    #[allow(unused)]
    pub fn remove_property(&mut self, property: FmProperty) {
        self.raw_file.remove_property(property.get_key());
    }

}

impl PartialEq for MdFile {
    fn eq(&self, other: &Self) -> bool {
        // TODO: should probably check if file inodes are the same instead
        // or otherwise ask the OS to check the paths for me
        self.path == other.path
    }
}

impl Eq for MdFile {}

impl FmProperty {
    pub fn get_key(&self) -> String {
        match &self {
            FmProperty::Inbox    => "inbox"   .to_owned(),
            FmProperty::Category => "category".to_owned(),
            FmProperty::Status   => "status"  .to_owned(),
            FmProperty::Type     => "type"    .to_owned(),
            FmProperty::Action   => "action"  .to_owned(),
        }
    }
}
