use std::path::PathBuf;

use yaml_serde::{Mapping, Value};

use crate::vault::{
    ecs::{Ecs, File, FileId, components::*}, fm::{parsing::{FrontmatterYaml, Parsed, parse_md_file}, *}
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

            self.add_component(file.id, FmYamlComponent(FmProp::new_unmodified(fm)));

            parsed.type_  .map(|type_|   self.add_component(file.id, FmTypeComponent   (FmProp::new_unmodified(type_))));
            parsed.action .map(|action|  self.add_component(file.id, FmActionComponent (FmProp::new_unmodified(action))));
            parsed.status .map(|status|  self.add_component(file.id, FmStatusComponent (FmProp::new_unmodified(status))));
            parsed.project.map(|project| self.add_component(file.id, FmProjectComponent(FmProp::new_unmodified(project))));
            parsed.tags   .map(|tags|    self.add_component(file.id, FmTagsComponent   (FmProp::new_unmodified(tags))));

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
