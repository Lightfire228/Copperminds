
use std::{fs, path::{Path, PathBuf}};

use crate::vault::{file_utilities::RawFile, fm::{FmProperty, GetKey}};

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
    pub fn needs_type(&self) -> bool {
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
    pub fn needs_action_type(&self) -> bool {
        self.is_actionable()
        && self.get_property(FmProperty::Action).is_none()
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

    pub fn is_property(&self, property: FmProperty, val: impl GetKey) -> bool {
        self.get_property(property).is_some_and(|p| p == val.get_key())
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
    use std::{path::Path};


    use crate::vault::fm::FmType;

    use super::*;


    fn load_file(name: &str) -> MdFile {
        let dir  = format!("{}/test_files/{name}", env!("CARGO_MANIFEST_DIR"));
        let path = Path::new(&dir).to_path_buf();

        MdFile::new(0, path)
    }


    #[test]
    fn test_type_sorting() {
        let untyped = load_file("sorting/type_none.md");
        let info    = load_file("sorting/type_info.md");
        let action  = load_file("sorting/type_action.md");

        assert_eq!(untyped.needs_type(), true);
        assert_eq!(info   .needs_type(), false);
        assert_eq!(action .needs_type(), false);

        assert_eq!(untyped.needs_action_type(), false);
        assert_eq!(info   .needs_action_type(), false);
        assert_eq!(action .needs_action_type(), true);

        assert_eq!(untyped.is_actionable(), false);
        assert_eq!(info   .is_actionable(), false);
        assert_eq!(action .is_actionable(), true);

        assert!(info  .is_property(FmProperty::Type, FmType::Info));
        assert!(action.is_property(FmProperty::Type, FmType::Action));
    }

    #[test]
    fn test_status_sorting() {
        let archive   = load_file("sorting/status_archive.md");
        let archived  = load_file("sorting/status_archived.md");
        let complete  = load_file("sorting/status_complete.md");
        let completed = load_file("sorting/status_completed.md");

        assert_eq!(archive  .is_archived(), true);
        assert_eq!(archived .is_archived(), true);
        assert_eq!(complete .is_archived(), false);
        assert_eq!(completed.is_archived(), false);

        assert_eq!(archive  .is_complete(), false);
        assert_eq!(archived .is_complete(), false);
        assert_eq!(complete .is_complete(), true);
        assert_eq!(completed.is_complete(), true);
    }

}
