use std::fs;

use yaml_serde::Mapping;

use crate::vault::{EcsFileView, ecs::{Ecs, File, FileId, FmComponent, parse::is_empty}};




impl Ecs {

    pub fn write_to_disk(&mut self, id: FileId) {

        let file = self.get(id).unwrap();
        let text = file.to_file_text();
        let file = file.file;

        // MAYBE: lock the file before writing to it?
        file.assert_unmodified();
        fs::write(&file.path, &text).unwrap();
    }



}

impl File {
    fn assert_unmodified(&self) {
        let text = fs::read_to_string(&self.path).unwrap();

        assert_eq!(self.raw_text, text, "there's a disturbance in the force");
    }

}
