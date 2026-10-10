use std::fs;


use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{Ecs, EcsFileView, File, FileId, components::{EmptyFileComponent, FmComponent, FmProp}}, fm::{FmProperty, GetKey}};




impl Ecs {

    pub fn write_to_disk(&mut self, id: FileId) {

        self.replicate_changes_to_fm(id);

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



impl<'a> EcsFileView<'a> {

    pub fn to_file_text(&self) -> String {
        let md = self.get_md_text();

        let Some(FmComponent { fm, .. }) = self.fm else {
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
