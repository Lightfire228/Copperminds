
use std::{fs, path::Path};

use super::regex;
use yaml_serde::{Mapping, Value};

const RE_EMPTY: &str = r"^\s*$";

/// This is only concerned with manipulating the raw text and files, and front matter dictionaries.
/// It does not provide any structured information about frontmatter properites, their state, or their validity
pub struct RawFile {
    pub frontmatter:      Option<Mapping>,

    pub md_text:          String,
    pub fm_text:          String,
}

impl RawFile {

    pub fn new(text: String) -> Self {
        parse_md_file(text)
    }

    pub fn write(&self, path: &Path) {
        let text = self.to_file_text();

        fs::write(path, &text).unwrap();
    }

    fn to_file_text(&self) -> String {
        let Some(fm) = &self.frontmatter else {
            return self.md_text.clone();
        };

        format!("---\n{}---\n{}", fm_to_text(&fm), self.md_text)
    }

    pub fn is_empty(&self) -> bool {
        self.is_md_empty() && self.is_fm_empty()
    }

    pub fn is_md_empty(&self) -> bool {
        regex!(RE = RE_EMPTY);

        RE.is_match(&self.md_text)
    }

    pub fn is_fm_empty(&self) -> bool {
        regex!(RE = RE_EMPTY);

        RE.is_match(&self.fm_text)
    }


    // ------- Crud ----------

    pub fn get_property(&self, property: &str) -> Result<String, PropertyError> {
        let fm = self.frontmatter.as_ref().ok_or(PropertyError::PropertyNotFound)?;

        get_val_by_name(fm, property)
    }

    pub fn get_property_as_list(&self, property: &str) -> Result<Vec<String>, PropertyListError> {
        let fm = self.frontmatter.as_ref().ok_or(PropertyListError::PropertyNotFound)?;

        get_val_list_by_name(fm, property)
    }

    pub fn set_property(&mut self, property: String, value: String) {

        let fm = self.frontmatter.get_or_insert_with(Mapping::new);

        let key = Value::String(property);
        let val = Value::String(value);

        fm.insert(key, val);
    }

    // pub fn push_property_list(&mut self, list: String, value: String) {

    //     let     fm   = self.frontmatter.get_or_insert_with(Mapping::new);
    //     let mut tags = fm.take_property_list(list);

    //     tags.push(value.clone());

    //     let key = Value::String  (list.get_key());
    //     let val = Value::Sequence(Sequence::from_iter(tags
    //         .iter()
    //         .map (|t| Value::String(t.to_owned()))
    //     ));

    //     fm.yaml.insert(key, val);
    //     fm.set_property_list(list, tags);
    // }


    pub fn remove_property(&mut self, property: String) {

        let Some(fm) = &mut self.frontmatter else {
            return;
        };

        fm.remove(Value::String(property));
    }

    // pub fn rename_property(&mut self, old_prop: FmProperty, new_prop: FmProperty) -> bool {

    //     let Some(fm) = &mut self.frontmatter else {
    //         return false;
    //     };

    //     if let Some(existing) = fm.get_property(new_prop) {
    //         panic!("{}: {} is already defined for file: {}", new_prop.get_key(), existing, self.file_name);
    //     }

    //     let Some(old_val) = fm.take_property(old_prop) else {
    //         return false;
    //     };

    //     fm.yaml.remove(old_prop.get_key());

    //     self.assign_property(new_prop, old_val);

    //     true
    // }

    // ----- /CRUD ----

}

fn parse_md_file(text: String) -> RawFile {


    let blank = |text: String| {
        RawFile {
            frontmatter: None,
            md_text:     text.clone(),
            fm_text:     String::new(),
        }
    };

    let parsed = get_frontmatter_text(text);

    let parsed = match parsed {
        Parsed::None       (text)      => return blank(text),
        Parsed::Frontmatter(parsed_fm) => parsed_fm
    };

    let Some(fm)   = yaml_serde::from_str::<Mapping>(&parsed.fm).ok() else {
        return blank(parsed.raw_text);
    };

    RawFile {
        frontmatter: Some(fm),
        md_text:     parsed.body,
        fm_text:     parsed.fm,
    }
}


fn get_frontmatter_text(text: String) -> Parsed {

    // (?ms) set flags
    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    //
    // - obsidian doesn't consider spaces after a --- fence to be a valid frontmatter section
    //   and \n matches crlf
    // - obsidian *does* consider an empty fm to be valid
    regex!(RE = r"^(?ms)---\n(.*?)\n---\n(.*)");

    let Some(captures) = RE.captures(&text) else {
        return Parsed::None(text);
    };

    let fm   = captures[1].to_owned();
    let body = captures[2].to_owned();

    Parsed::Frontmatter(ParsedFm {
        raw_text: text,
        fm,
        body,
    })

}

enum Parsed {
    None       (String),
    Frontmatter(ParsedFm),
}

struct ParsedFm {
    pub raw_text: String,
    pub fm:       String,
    pub body:     String,
}

fn get_val_by_name(fm: &Mapping, name: &str) -> Result<String, PropertyError> {
    // TODO: as_str doesn't coerce bools, numbers, or single element lists into strings

    let prop = fm.get(name) .ok_or(PropertyError::PropertyNotFound)?;
    let val  = prop.as_str().ok_or(PropertyError::ValueNotFound)?;

    Ok(val.to_owned())
}

fn get_val_list_by_name(fm: &Mapping, name: &str) -> Result<Vec<String>, PropertyListError> {

    let prop  = fm.get(name).ok_or(PropertyListError::PropertyNotFound)?;
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


pub enum PropertyError {
    PropertyNotFound,
    ValueNotFound,
}

pub enum PropertyListError {
    PropertyNotFound,
}


fn fm_to_text(fm: &Mapping) -> String {
    yaml_serde::to_string(fm).unwrap()
}

#[cfg(test)]
mod tests {
    use std::{env, vec};

    use super::*;

    fn load_file(name: &str) -> String {
        let dir  = format!("{}/test_files/parsing/{name}", env!("CARGO_MANIFEST_DIR"));
        let path = Path::new(&dir);

        fs::read_to_string(path).unwrap()
    }

    fn load_test_bodies() -> Vec<String> {
        vec![
            load_file("test_body_01.md"),
            load_file("test_body_02.md"),
        ]
    }

    #[test]
    fn test_parsing_no_fm() {

        let bodies = load_test_bodies();

        let invalid_fm = vec![
            load_file("invalid_fm_01.yaml"),
            load_file("invalid_fm_02.yaml"),
            load_file("invalid_fm_03.yaml"),
            load_file("invalid_fm_04.yaml"),
        ];

        let mut tests = Vec::new();

        for fm in invalid_fm {
            for body in bodies.iter() {

                tests.push(format!("---\n{}\n---\n{body}", fm.trim()));
            }
        }

        // test files without an explicit front matter
        tests.extend(bodies);


        for text in tests {
            let parsed = parse_md_file(text.clone());

            assert_eq!(parsed.frontmatter, None);
            assert_eq!(parsed.fm_text,     "");
            assert_eq!(parsed.md_text,     text);
        }
    }

    #[test]
    fn test_parsing_fm() {

        let bodies = load_test_bodies();

        let fms = vec![
            load_file("valid_fm_01.yaml"),
            load_file("valid_fm_01.yaml"),

            // test empty front matter
            "".to_string(),
        ];

        for fm in fms {
            for body in bodies.iter() {

                let fm = fm.trim();

                let text   = format!("---\n{fm}\n---\n{body}");
                let parsed = parse_md_file(text.clone());

                assert_ne!(parsed.frontmatter, None);
                assert_eq!(parsed.md_text,     *body);
                assert_eq!(parsed.fm_text,     fm);
            }
        }
    }


    #[test]
    fn base() {
        // - is empty file
        // - properties
        //   - add
        //   - remove
        //   - overwrite
        //
        // make sure to check for data loss
        //

    }

}
