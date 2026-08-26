use std::fmt::Debug;

use crate::vault::md_file::FileView;

/// this just reduces logging noise for message events
#[repr(transparent)]
pub struct Files {
    files: Vec<FileView>
}

impl From<Vec<FileView>> for Files {
    fn from(val: Vec<FileView>) -> Self {
        Files { files: val }
    }
}
impl From<Files> for Vec<FileView> {
    fn from(val: Files) -> Self {
        val.files
    }
}

impl Debug for Files {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Files]")
    }
}


impl Files {
    pub fn to_list(self) -> Vec<FileView> {
        self.into()
    }
}
