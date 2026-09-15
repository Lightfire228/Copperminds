
use std::{fs, path::{Path, PathBuf}};

use file_id::FileId;

use crate::{file_shit, vault::{EcsFileView, file_utilities::RawFile, fm::{FmProperty, FmType, GetKey}, watch::FileData}};

use super::build_regex;

#[derive(Debug)]
struct MdFile {
    pub id:               FileId,
    pub path:             PathBuf,
    pub raw_file:         RawFile,

    /// Includes file extension
    pub file_name:        String,
}

impl MdFile {

    pub fn new(data: FileData) -> Self {
        let text = file_shit::get_file_text(&data.name);

        Self {
            id:        data.id,
            file_name: file_shit::get_file_name(&data.name),
            path:      data.name,
            raw_file:  RawFile::new(text),
        }
    }

    // --- queries

    pub fn is_empty_parsed(&self) -> bool {
        self.raw_file.is_empty_parsed()
    }

    pub fn is_empty_raw(&self) -> bool {
        self.raw_file.is_empty_raw()
    }

    pub fn is_md_empty(&self) -> bool {
        self.raw_file.is_md_empty()
    }

    pub fn is_unnamed(&self) -> bool {
        // (?i) - sets case insensitivity
        build_regex!(RE = r"(?i)^([\d \-_]*|Untitled(\s.*?)?)\.md$");

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
            // TODO: move this to a config
            .any      (|p| p.ends_with("03 Data"))
    }

    pub fn needs_sorting(&self) -> bool {
        [
            self.needs_type           (),
            self.needs_action_assigned(),
            self.is_unnamed           (),
        ]
            .iter()
            .any (|x| *x)


    }
    /// returns true if the file has type of action, and doesn't have an action assigned
    pub fn needs_action_assigned(&self) -> bool {
           self.is_type_action()
        && self.get_property(FmProperty::Action).is_none()
    }

    /// returns true if the file has type of action, and has an action assigned
    /// does not account for Complete or Archived status
    pub fn is_actionable(&self) -> bool {
           self.is_type_action()
        && self.get_property(FmProperty::Action).is_some()
    }

    pub fn is_type_info(&self) -> bool {
        self.is_property(FmProperty::Type, FmType::Info)
    }

    pub fn is_type_action(&self) -> bool {
        self.is_property(FmProperty::Type, FmType::Action)
    }

    /// not completed nor archived
    pub fn is_open(&self) -> bool {
        !(self.is_complete() || self.is_archived())
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


    pub fn write_file(&mut self) {
        self.raw_file.write(&self.path);
    }


    pub fn set_property(&mut self, property: FmProperty, value: String) {
        self.raw_file.set_property(property.get_key(), value);
    }

    pub fn remove_property(&mut self, property: FmProperty) {
        self.raw_file.remove_property(property.get_key());
    }

}
