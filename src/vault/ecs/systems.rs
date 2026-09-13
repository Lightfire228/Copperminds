use std::path::Path;

use file_id::FileId;

use crate::vault::{ecs::{Ecs, FileView}};



pub fn write_to_disk(ecs: &mut Ecs, id: FileId) {

}

impl Ecs {

    fn get_empty_unnamed_files(&self) -> impl Iterator<Item = FileView<'_>> {
        self
            .get_all()
            .filter(|f| f.is_empty && f.is_unnamed())
    }

    pub fn delete_empty_unnamed_files(&mut self) {

        let files: &[FileId] = todo!();

        for id in files {
            let file = self.remove_file(id);

            trash::delete(file.path).unwrap();
        }
    }

}
