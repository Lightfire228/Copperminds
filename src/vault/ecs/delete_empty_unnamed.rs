use crate::{file_shit, vault::ecs::{Ecs, EcsFileView}};


impl Ecs {

    pub fn get_empty_unnamed_files(&self) -> impl Iterator<Item = EcsFileView<'_>> {
        self
            .get_all()
            .filter(|f| f.is_empty() && f.is_unnamed())
    }

    pub fn delete_empty_unnamed_files(&mut self) {

        let files: Vec<_> = self.get_empty_unnamed_files().map(|f| f.id).collect();

        for id in files {
            let file = self.remove_file(id);

            file_shit::delete_from_disk(&file.path);
        }
    }

}




#[cfg(test)]
mod tests {
    use std::{path::PathBuf};

    use crate::{test_utils::id, vault::ecs::{FileId, NewFile}};

    use super::*;

    fn build_empty_unnamed_test_cases(
        empty_titles:     &[String],
        non_empty_titles: &[String],
        non_empty_bodies: &[&str],
    )
        -> Vec<Test>
    {
        let mut test_cases = vec![];

        macro_rules! push {
            ($title:expr, $body:expr, $should_delete:expr) => {
                test_cases.push(Test {
                    id:            id(),
                    file_name:     $title.to_owned(),
                    file_body:     $body .to_owned(),
                    should_delete: $should_delete,
                })
            };
        }

        macro_rules! should_delete {
            ($title:expr, $body:expr) => {
                push!($title, $body, true);
            };
        }
        macro_rules! should_not_delete {
            ($title:expr, $body:expr) => {
                push!($title, $body, false);
            };
        }

        for empty_title in empty_titles.iter() {
            should_delete!(empty_title, "");
            should_delete!(empty_title, " \t\n");

            for body in non_empty_bodies.iter() {
                should_not_delete!(empty_title, *body);
            }

        }
        for non_empty_title in non_empty_titles.iter() {
            should_not_delete!(non_empty_title, "");
            should_not_delete!(non_empty_title, " \t\n");

            for body in non_empty_bodies.iter() {
                should_not_delete!(non_empty_title, *body);
            }
        }

        test_cases
    }




    #[derive(Debug)]
    struct Test {
        id:            FileId,
        file_name:     String,
        file_body:     String,
        should_delete: bool,
    }

    /// since `delete_empty_unnamed_files` runs automatically on startup,
    /// it's critical to make sure it doesn't delete anything remotely important
    #[test]
    fn test_empty_unnamed_files() {
        let empty_titles = [
            "Untitled",
            "untitled",
            "Untitled - 1",
            "Untitled (2)",
            "2026-01-01",
            "2026-01-01 ",
            "2026-01-01 - 00_00_00",
            "2026-01-01 - 00",
            "2026",
            "1",
            "___ ---",
        ]
            .map(|f| format!("{f}.md"))
        ;

        let non_empty_titles = [
            "Untitledtropolis",
            "Untitled-thingy",
            "Untitled Goose Game",
            "2026-01-01 - 00_00_00 - titled",
            "2026-01-01 - titled",
            "2026-01-01 titled",
            "2026-01 there are rats in my basement",
            "dorktastic",
        ]
            .map(|f| format!("{f}.md"))
        ;

        let non_empty_bodies = [
            "---\n\n---\n",
            ".",
        ];

        let test_cases = build_empty_unnamed_test_cases(&empty_titles, &non_empty_titles, &non_empty_bodies);


        let mut ecs = Ecs::default();

        test_cases
            .iter    ()
            .for_each(|t| ecs.new_file(NewFile {
                id:       t.id,
                path:     PathBuf::new(),
                raw_text: t.file_body.clone(),
                name:     t.file_name.clone(),
            }))
        ;

        let empty_unamed: Vec<_> = ecs
            .get_empty_unnamed_files()
            .collect()
        ;

        for file in test_cases {

            let found = empty_unamed.iter().any(|x| x.id == file.id);

            assert_eq!(found, file.should_delete, "{file:?}")
        }

    }

}
