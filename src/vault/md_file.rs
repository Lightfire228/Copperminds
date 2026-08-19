
use std::{fs, path::{Path, PathBuf}};

use file_id::FileId;

use crate::vault::{file_utilities::RawFile, fm::{FmProperty, FmType, GetKey}, watch::FileData};

use super::regex;

#[derive(Debug)]
pub struct MdFile {
    pub id:               FileId,
    pub path:             PathBuf,
    pub raw_file:         RawFile,

    /// Includes file extension
    pub file_name:        String,
}

#[derive(Debug, Clone, Eq)]
pub struct FileView {
    pub id:   FileId,
    pub name: String,
}


impl MdFile {

    pub fn new(data: FileData) -> Self {
        let text = fs::read_to_string(&data.name).unwrap();

        Self {
            id:        data.id,
            file_name: data.name.file_name().unwrap().to_str().unwrap().to_owned(),
            path:      data.name,
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
    pub fn needs_type(&self) -> bool {
        self.get_property(FmProperty::Type).is_none()
        && !self.path
            .ancestors()
            .any      (|p| p.ends_with("03 Data"))
    }

    /// returns true if the file has type of action, and doesn't have an action assigned
    pub fn needs_action_type(&self) -> bool {
           self.is_property (FmProperty::Type, FmType::Action)
        && self.get_property(FmProperty::Action).is_none()
    }

    /// returns true if the file has type of action, and has an action assigned
    pub fn is_actionable(&self) -> bool {
           self.is_property (FmProperty::Type, FmType::Action)
        && self.get_property(FmProperty::Action).is_some()
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

    pub fn is_property(&self, property: FmProperty, val: impl GetKey) -> bool {
        self.get_property(property).is_some_and(|p| p == val.get_key())
    }

    pub fn is_property_any_of(&self, property: FmProperty, vals: &[impl GetKey + Eq]) -> bool {
        let vals: Vec<_> = vals
            .iter   ()
            .map    (|p| p.get_key())
            .collect()
        ;

        self
            .get_property(property)
            .is_some_and (|p|
                vals.contains(&p)
            )
    }

    // ---- writes


    pub fn refresh(&mut self) {
        let data = (&*self).into();
        *self = Self::new(data);
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
        // MAYBE: should probably check if file inodes are the same instead
        // or otherwise ask the OS to check the paths for me
        self.path == other.path
    }
}

impl Eq for MdFile {}


impl PartialEq for FileView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<&MdFile> for FileView {
    fn from(value: &MdFile) -> Self {
        FileView {
            id:   value.id,
            name: value.file_name.clone(),
        }
    }
}

impl From<MdFile> for FileData {
    fn from(val: MdFile) -> Self {
        FileData {
            id:   val.id,
            name: val.path,
        }
    }
}

impl From<&MdFile> for FileData {
    fn from(val: &MdFile) -> Self {
        FileData {
            id:   val.id,
            name: val.path.clone(),
        }
    }
}


#[cfg(test)]
impl MdFile {
    pub fn parse(id: FileId, text: String) -> Self {
        Self {
            id,
            file_name: String ::new(),
            path:      PathBuf::new(),
            raw_file:  RawFile::new(text),
        }
    }
}


#[cfg(test)]
mod tests {

    use yaml_serde::{Mapping, Value};

    use crate::vault::{file_utilities::PropertyError, fm::{FmAction, FmStatus, FmType}};

    use super::*;


    fn mapping_to_str(fm: Mapping) -> String {
        format!("---\n{}---\n", yaml_serde::to_string(&fm).unwrap())
    }

    fn id() -> FileId {
        FileId::Inode {
            device_id:    0,
            inode_number: 0,
        }
    }


    macro_rules! fm {
        ( $($key:expr => $value:expr),*$(,)? ) => {{
            #[allow(unused_mut)]
            let mut fm = Mapping::new();

            $(
                fm.insert(Value::String($key.get_key()), Value::String($value.get_key()));
            )*

            MdFile::parse(id(), mapping_to_str(fm))
        }};
    }

    fn from_yaml(text: &str) -> MdFile {
        let yaml: Mapping = yaml_serde::from_str(text).unwrap();

        let text = mapping_to_str(yaml);
        MdFile::parse(id(), text)
    }

    #[test]
    fn test_type_sorting() {

        let untyped = fm!();
        let info    = fm!(FmProperty::Type => FmType::Info);
        let action  = fm!(FmProperty::Type => FmType::Action);

        assert_eq!(untyped.needs_type(), true);
        assert_eq!(info   .needs_type(), false);
        assert_eq!(action .needs_type(), false);

        assert!(info  .is_property(FmProperty::Type, FmType::Info));
        assert!(action.is_property(FmProperty::Type, FmType::Action));
    }

    #[test]
    fn test_action_sorting() {

        let no_action_info     = fm!(FmProperty::Type => FmType::Info);
        let no_action          = fm!(                                    FmProperty::Action => FmAction::Todo);
        let needs_action       = fm!(FmProperty::Type => FmType::Action);
        let action_todo        = fm!(FmProperty::Type => FmType::Action, FmProperty::Action => FmAction::Todo);
        let action_waiting_for = fm!(FmProperty::Type => FmType::Action, FmProperty::Action => FmAction::WaitingFor);

        assert_eq!(no_action_info    .needs_action_type(), false);
        assert_eq!(no_action         .needs_action_type(), false);
        assert_eq!(needs_action      .needs_action_type(), true);
        assert_eq!(action_todo       .needs_action_type(), false);
        assert_eq!(action_waiting_for.needs_action_type(), false);

        assert_eq!(no_action_info    .is_actionable(),     false);
        assert_eq!(no_action         .is_actionable(),     false);
        assert_eq!(needs_action      .is_actionable(),     false);
        assert_eq!(action_todo       .is_actionable(),     true);
        assert_eq!(action_waiting_for.is_actionable(),     true);
    }

    #[test]
    fn test_status_sorting() {
        let archive   = fm!(FmProperty::Status => FmStatus::Archive);
        let archived  = fm!(FmProperty::Status => FmStatus::Archived);
        let complete  = fm!(FmProperty::Status => FmStatus::Complete);
        let completed = fm!(FmProperty::Status => FmStatus::Completed);

        assert_eq!(archive  .is_archived(), true);
        assert_eq!(archived .is_archived(), true);
        assert_eq!(complete .is_archived(), false);
        assert_eq!(completed.is_archived(), false);

        assert_eq!(archive  .is_complete(), false);
        assert_eq!(archived .is_complete(), false);
        assert_eq!(complete .is_complete(), true);
        assert_eq!(completed.is_complete(), true);
    }

    #[test]
    fn test_property_coercion() {
        let test = from_yaml(r#"
            bool:   true
            number: 42
            single:
                - thingy

            empty: []
            many:
                - thingy 1
                - thingy 2

            map:
                a: b
                b: c
        "#);


        assert_eq!(test.raw_file.get_property("bool")  .unwrap(), "true");
        assert_eq!(test.raw_file.get_property("number").unwrap(), "42");
        assert_eq!(test.raw_file.get_property("single").unwrap(), "thingy");

        assert_eq!(test.raw_file.get_property("empty").unwrap_err(), PropertyError::ValueNotFound);
        assert_eq!(test.raw_file.get_property("many") .unwrap_err(), PropertyError::PropertyIsList);
        assert_eq!(test.raw_file.get_property("map")  .unwrap_err(), PropertyError::PropertyIsMapping);
    }

}
