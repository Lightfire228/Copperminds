use std::fs;


use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{Components, Ecs, EcsFileView, File, FileId, components::{EmptyFileComponent, FmComponent, FmProp, FmYamlComponent}}, fm::{FmProperty, GetKey}};




impl Components {

    pub fn write_to_disk(&mut self, id: FileId, file: &File) {

        self.replicate_changes_to_fm(id);

        let file = self.to_file_view(id, file);
        let text = file.to_file_text();
        let file = file.file;

        // MAYBE: lock the file before writing to it?
        file.assert_unmodified();
        fs::write(&file.path, &text).unwrap();
    }
}

impl Ecs {
    pub fn write_to_disk(&mut self, id: FileId) {
        let Some(file) = self.file.get(&id) else {
            return;
        };

        self.components.write_to_disk(id, file);
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

        let Some(FmYamlComponent(fm)) = self.fm else {
            return md;
        };

        let Some(fm) = &fm.value else {
            return md;
        };


        format!("---\n{}---\n{}", fm_to_text(fm), md)
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
