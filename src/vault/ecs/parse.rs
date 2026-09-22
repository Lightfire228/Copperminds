use yaml_serde::{Mapping, Value};

use crate::vault::{
    ecs::{Ecs, File, FileId, NewFile, components::*},
    fm::*
};

use super::super::build_regex;
use crate::prelude::*;


const RE_EMPTY: &str = r"^\s*$";

pub struct Parsed {
    pub fm: Option<Mapping>,
    pub md: String,
}

struct Extract {
    pub fm: Option<String>,
    pub md: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyError {
    PropertyNotFound,
    ValueNotFound,
    PropertyIsList,
    PropertyIsMapping,
    PropertyIsTagged,
}

pub enum _PropertyListError {
    PropertyNotFound,
}

macro_rules! parse_prop {
    ($fm:expr, $kind:expr) => { (|| {
        let Ok(x) = fm_get_property($fm, $kind) else {
            None?
        };

        #[allow(irrefutable_let_patterns)]
        let Ok  (x) = x.as_str().try_into() else {
            warn!("unknown {} prop: {x}", $kind.get_key());
            None?
        };

        Some(x)
    })()};
}

macro_rules! parse_or_bail {
    ($fm:ident, $prop:expr) => {{
        let Some(x) = parse_prop!($fm, $prop) else {
            return;
        };

        x
    }};
}



impl Ecs {

    pub fn new_file(&mut self, file: NewFile) {

        let is_empty = is_empty(&file.raw_text);

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

            if let Some(type_) = parse_prop!(&fm, FmProperty::Type) {
                self.parse_info(type_, file.id);

                self.add_component(file.id, TypeComponent { type_ });
            }

            self.add_action (&fm, file.id);
            self.add_status (&fm, file.id);
            self.add_project(&fm, file.id);


            self.add_component(file.id, FmComponent { fm });
        }

        if is_empty {
            self.add_component(file.id, EmptyComponent);
        }

        self.add_component(file.id, MdTextComponent { text: md });
    }

    fn parse_info(&mut self, type_: FmType, id: FileId) {
        if !matches!(type_, FmType::Info) {
            return;
        }

        self.add_component(id, InfoComponent);
    }


    fn add_action(&mut self, fm: &Mapping,id: FileId) {
        let action = parse_or_bail!(fm, FmProperty::Action);

        self.add_component(id, ActionComponent {
            action,
        });
    }

    fn add_status(&mut self, fm: &Mapping,id: FileId) {
        let status = parse_or_bail!(fm, FmProperty::Status);

        self.add_component(id, StatusComponent {
            status,
        });
    }

    fn add_project(&mut self, fm: &Mapping, id: FileId){
        let project = parse_or_bail!(fm, FmProperty::Project);

        self.add_component(id, ProjectComponent {
            project,
        });
    }

}

pub fn parse_md_file(text: &str) -> Parsed {

    let blank = || {
        return Parsed {
            fm:      None,
            md:      text.to_string(),
        }
    };

    let extract = extract_frontmatter_text(text);

    let Some(fm) = extract.fm else {
        return blank();
    };

    let Some(fm)   = yaml_serde::from_str::<Mapping>(&fm).ok() else {
        return blank();
    };

    Parsed {
        fm: Some(fm),
        md: extract.md,
    }
}

fn extract_frontmatter_text(text: &str) -> Extract {

    // (?ms) set flags
    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    //
    // - obsidian doesn't consider spaces after a --- fence to be a valid frontmatter section
    //   and \n matches crlf
    // - obsidian *does* consider an empty fm to be valid
    build_regex!(RE = r"^(?ms)---\n(.*?)\n---\n(.*)");

    let Some(captures) = RE.captures(&text) else {
        return Extract {
            fm: None,
            md: text.to_string(),
        }
    };

    let fm   = captures[1].to_string();
    let body = captures[2].to_string();

    Extract {
        fm: Some(fm),
        md: body,
    }

}


fn fm_get_property(fm: &Mapping, prop: FmProperty) -> Result<String, PropertyError> {
    yaml_get_val_by_name(fm, &prop.get_key())
}

fn yaml_get_val_by_name(fm: &Mapping, name: &str) -> Result<String, PropertyError> {
    let prop = fm.get(name) .ok_or(PropertyError::PropertyNotFound)?;

    yaml_val_to_str(prop)
}

fn yaml_val_to_str(value: &Value) -> Result<String, PropertyError> {

    Ok(match value {
        Value::Bool    (x) => x.to_string(),
        Value::Number  (x) => x.to_string(),
        Value::String  (x) => x.to_owned (),
        Value::Sequence(x) => {

            match x.len() {
                1 => yaml_val_to_str(&x[0])?,

                0 => Err(PropertyError::ValueNotFound)?,
                _ => Err(PropertyError::PropertyIsList)?,
            }
        }

        Value::Mapping(_) => Err(PropertyError::PropertyIsMapping)?,
        Value::Tagged (_) => Err(PropertyError::PropertyIsTagged)?,
        Value::Null       => Err(PropertyError::ValueNotFound)?,
    })
}


fn _yaml_get_val_list_by_name(fm: &Mapping, name: &str) -> Result<Vec<String>, _PropertyListError> {

    let prop  = fm.get(name).ok_or(_PropertyListError::PropertyNotFound)?;
    let empty = vec![];

    // this doesn't catch tagged values, but i don't use those so /shrug
    let values = match prop {
        Value::Null             => &empty,
        Value::Sequence(values) => values,
        _ => panic!("value is not a list")
    };

    let values: Vec<_> = values
        .into_iter ()
        .filter_map(|v| v
            .as_str()
            .map   (|v| v.to_owned())
        )
        .collect   ()
    ;

    Ok(values)
}


pub fn is_unnamed(file_name: &str) -> bool {
    // (?i) - sets case insensitivity
    build_regex!(RE = r"(?i)^([\d \-_]*|Untitled(\s.*?)?)\.md$");

    RE.is_match(file_name)
}


pub fn is_empty(file_name: &str) -> bool {
    build_regex!(RE = RE_EMPTY);

    RE.is_match(file_name)
}






/// The main concern here is avoiding data loss.
/// Nothing should change the file in any way other than the intended effect
#[cfg(test)]
mod tests {
    use std::{path::{PathBuf}, vec};

    use crate::test_utils::{id, load_file};

    use super::*;


    fn load_test_bodies() -> Vec<String> {
        vec![
            load_file("parsing/test_body_01.md"),
            load_file("parsing/test_body_02.md"),
        ]
    }

    #[test]
    fn test_parsing_no_fm() {

        let bodies = load_test_bodies();

        let invalid_fm = vec![
            load_file("parsing/invalid_fm_01.yaml"),
            load_file("parsing/invalid_fm_02.yaml"),
            load_file("parsing/invalid_fm_03.yaml"),
            load_file("parsing/invalid_fm_04.yaml"),
        ];

        struct Test {
            body: String,
            fm:   Option<String>,
        }

        let mut tests = Vec::new();

        for fm in invalid_fm {
            let fm = fm.trim();

            for body in bodies.iter() {

                tests.push(Test {
                    body: format!("---\n{fm}\n---\n{body}"),
                    fm:   Some(fm.to_string()),
                });
            }
        }

        tests.extend(bodies.into_iter().map(|x| Test {
            body: x,
            fm:   None,
        }));


        for test in tests {
            let extract = extract_frontmatter_text(&test.body);
            let parsed  = parse_md_file(&test.body);

            assert_eq!(parsed .fm, None);
            assert_eq!(parsed .md, test.body);

            assert_eq!(extract.fm, test.fm);

        }
    }

    #[test]
    fn test_parsing_fm() {

        let bodies = load_test_bodies();

        let fms = vec![
            load_file("parsing/valid_fm_01.yaml"),
            load_file("parsing/valid_fm_01.yaml"),

            // test empty front matter
            "".to_string(),
        ];

        for fm in fms {
            for body in bodies.iter() {

                let fm = fm.trim();

                let text    = format!("---\n{fm}\n---\n{body}");
                let extract = extract_frontmatter_text(&text);
                let parsed  = parse_md_file(&text);

                assert_ne!(parsed .fm, None);
                assert_eq!(parsed .md, *body);

                assert_eq!(extract.fm.unwrap(), fm);
            }
        }
    }

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

    #[test]
    #[ignore = "TODO"]
    fn test_parsing_project() {
        todo!()
    }

    fn from_yaml(text: &str) -> Mapping {
        yaml_serde::from_str(text).unwrap()
    }


    #[test]
    fn test_property_coercion() {
        let test = from_yaml(r#"
            bool:   true
            number: 42
            single:
                - thingy

            empty: []
            many:
                - thingy 1
                - thingy 2

            map:
                a: b
                b: c
        "#);

        assert_eq!(yaml_get_val_by_name(&test, "bool")  .unwrap(), "true");
        assert_eq!(yaml_get_val_by_name(&test, "number").unwrap(), "42");
        assert_eq!(yaml_get_val_by_name(&test, "single").unwrap(), "thingy");

        assert_eq!(yaml_get_val_by_name(&test, "empty").unwrap_err(), PropertyError::ValueNotFound);
        assert_eq!(yaml_get_val_by_name(&test, "many") .unwrap_err(), PropertyError::PropertyIsList);
        assert_eq!(yaml_get_val_by_name(&test, "map")  .unwrap_err(), PropertyError::PropertyIsMapping);
    }


}
