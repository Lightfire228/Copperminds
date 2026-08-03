
use std::{fs, path::Path};

use super::regex;
use yaml_serde::{Mapping, Value};

const RE_EMPTY: &str = r"^\s*$";

/// This is only concerned with manipulating the raw text and files, and front matter dictionaries.
/// It does not provide any structured information about frontmatter properites, their state, or their validity
#[derive(Debug)]
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

    #[allow(unused)]
    pub fn get_property_as_list(&self, property: &str) -> Result<Vec<String>, PropertyListError> {
        let fm = self.frontmatter.as_ref().ok_or(PropertyListError::PropertyNotFound)?;

        get_val_list_by_name(fm, property)
    }

    pub fn set_property(&mut self, property: String, value: String) {

        let fm = self.frontmatter.get_or_insert_with(Mapping::new);

        let key = Value::String(property);
        let val = Value::String(value);

        fm.insert(key, val);
        self.fm_text = fm_to_text(fm);
    }

    pub fn remove_property(&mut self, property: String) {

        let Some(fm) = &mut self.frontmatter else {
            return;
        };

        fm.remove(Value::String(property));
    }

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
    let prop = fm.get(name) .ok_or(PropertyError::PropertyNotFound)?;

    Ok(val_to_str(prop)?)
}

fn val_to_str(value: &Value) -> Result<String, PropertyError> {

    Ok(match value {
        Value::Bool    (x) => x.to_string(),
        Value::Number  (x) => x.to_string(),
        Value::String  (x) => x.to_owned (),
        Value::Sequence(x) => {

            match x.len() {
                1 => val_to_str(&x[0])?,

                0 => Err(PropertyError::ValueNotFound)?,
                _ => Err(PropertyError::PropertyIsList)?,
            }
        }

        Value::Mapping(_) => Err(PropertyError::PropertyIsMapping)?,
        Value::Tagged (_) => Err(PropertyError::PropertyIsTagged)?,
        Value::Null       => Err(PropertyError::ValueNotFound)?,
    })
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyError {
    PropertyNotFound,
    ValueNotFound,
    PropertyIsList,
    PropertyIsMapping,
    PropertyIsTagged,
}

pub enum PropertyListError {
    PropertyNotFound,
}


fn fm_to_text(fm: &Mapping) -> String {
    yaml_serde::to_string(fm).unwrap()
}

/// The main concern here is avoiding data loss.
/// Nothing should change the file in any way other than the intended effect
#[cfg(test)]
mod tests {
    use std::{env, vec};

    use super::*;

    fn load_file(name: &str) -> String {
        let dir  = format!("{}/test_files/{name}", env!("CARGO_MANIFEST_DIR"));
        let path = Path::new(&dir);

        fs::read_to_string(path).unwrap()
    }

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
            load_file("parsing/valid_fm_01.yaml"),
            load_file("parsing/valid_fm_01.yaml"),

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
    fn test_empty() {

        let empty        = " \n\n\n\t\t\t\t    \t\t \n\n \t   ".to_owned();
        let not_empty_01 = format!("{}.{}", empty, empty);
        let not_empty_02 = format!("{}-{}", empty, empty);
        let not_empty_03 = format!("{}a{}", empty, empty);

        let tests = vec![
            (empty,        true),
            (not_empty_01, false),
            (not_empty_02, false),
            (not_empty_03, false),
        ];

        for (text, val) in tests {
            let parsed = parse_md_file(text);
            assert_eq!(parsed.is_empty(), val);
        }
    }

    #[test]
    fn test_property_writes() {
        macro_rules! value {
            ($x:literal) => {
                Value::String($x.to_owned())
            };
        }

        let bodies = load_test_bodies();

        for text in bodies {
            let mut body = parse_md_file(text.clone());
            let mut fm   = Mapping::new();

            macro_rules! assert {
                () => {
                    assert_eq!(body.frontmatter.as_ref(), Some(&fm));
                    assert_eq!(body.md_text,              text);
                };
            }

            macro_rules! add {
                ($key:literal, $val:literal) => {
                    body.set_property($key.to_owned(), $val.to_owned());
                    fm.insert(value!($key), value!($val));

                    assert!();
                };
            }

            add!("test prop 01", "test val 01");
            add!("test prop 02", "test val 02");
            add!("test prop 03", "test val 03");


            // test modify
            body.set_property("test prop 02".to_owned(), "changed".to_owned());

            let x = fm.get_mut(value!("test prop 02")).unwrap();
            *x = value!("changed");

            assert!();


            // test delete
            body.remove_property("test prop 02".to_owned());
            fm.remove(value!("test prop 02"));

            assert!();
        }
    }

}
