use std::path::PathBuf;

use yaml_serde::{Mapping, Value};

use crate::vault::{
    ecs::{Ecs, File, FileId, components::*}, fm::{parsing::{Parsed, parse_md_file}, *}
};

use super::super::build_regex;
use crate::prelude::*;


const RE_EMPTY: &str = r"^\s*$";

pub struct NewFile {
    pub id:       FileId,

    pub path:     PathBuf,
    pub raw_text: String,
    pub name:     String,
}

impl Ecs {

    // TODO: make this a plug and play pipeline
    pub fn new_file(&mut self, file: NewFile) {

        let is_empty   = is_text_empty   (&file.raw_text);
        let is_illegal = has_illegal_name(&file.name);


        let Parsed { fm, md } = parse_md_file(&file.raw_text);


        self.file.insert(file.id, File {
            unnamed:  is_unnamed(&file.name),
            name:     file.name,
            raw_text: file.raw_text,
            path:     file.path,
        });


        // TODO: get rid of the type property
        // it can be inferred from the other top level properties
        // - info
        //   - TODO: move the info from 'type' to a dedicated prop
        // - action

        if let Some(fm) = fm {

            let parsed: FrontmatterYaml = (&fm).into();

            let modified = false;
            self.add_component(file.id, FmComponent {
                fm,
                type_:    FmProp { value: parsed.type_,   modified },
                action:   FmProp { value: parsed.action,  modified },
                status:   FmProp { value: parsed.status,  modified },
                // project:  FmProp { value: parsed.project, modified },

                modified: false,
            });
        }

        if is_empty {
            self.add_component(file.id, EmptyFileComponent);
        }

        if is_illegal {
            self.add_component(file.id, IllegalNameComponent);
        }

        self.add_component(file.id, MdTextComponent { text: md });
    }

}




pub fn is_unnamed(file_name: &str) -> bool {
    // (?i) - sets case insensitivity
    build_regex!(RE = r"(?i)^([\d \-_]*|Untitled(\s.*?)?)\.md$");

    RE.is_match(file_name)
}


pub fn is_text_empty(text: &str) -> bool {
    build_regex!(RE = RE_EMPTY);

    RE.is_match(text)
}

pub fn has_illegal_name(file_name: &str) -> bool {
    build_regex!(RE = r#"[\?"<>\|:*\\/]"#);

    RE.is_match(file_name)
}


/// The main concern here is avoiding data loss.
/// Nothing should change the file in any way other than the intended effect
#[cfg(test)]
mod tests {
    use std::{path::{PathBuf}, vec};

    use crate::test_utils::{id, load_file};

    use super::*;


    #[test]
    fn test_empty() {

        let mut ecs = Ecs::default();

        let empty        = " \n\n\n\t\t\t\t    \t\t \n\n \t   ".to_owned();
        let not_empty_01 = format!("{}.{}", empty, empty);
        let not_empty_02 = format!("{}-{}", empty, empty);
        let not_empty_03 = format!("{}a{}", empty, empty);
        let not_empty_04 = format!("{}{}.", empty, empty);

        let tests = vec![
            (empty,        true),
            (not_empty_01, false),
            (not_empty_02, false),
            (not_empty_03, false),
            (not_empty_04, false),
        ];

        for (text, val) in tests {
            let id = id();

            ecs.new_file(NewFile {
                id,
                path:     PathBuf::new(),
                raw_text: text.to_owned(),
                name:     String::new(),
            });

            let file = ecs.get(id).unwrap();

            assert_eq!(file.is_empty(), val);
        }
    }

}
