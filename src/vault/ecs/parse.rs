use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{ActionComponent, Ecs, File, FileId, FmComponent, MdTextComponent, NewFile, StatusComponent, TypeComponent}, fm::{FmProperty, FmType, GetKey}};

use super::super::build_regex;
use crate::prelude::*;

const RE_EMPTY: &str = r"^\s*$";

pub struct Parsed {
    pub fm:       Option<Mapping>,
    pub md:       String,
    pub is_empty: bool,
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

pub enum PropertyListError {
    PropertyNotFound,
}



impl Ecs {

    pub fn new_file(&mut self, file: NewFile) {

        let Parsed { fm, md, is_empty } = parse_md_file(&file.raw_text);

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

            if let Some(type_) = self.add_type(&fm, file.id) {
                self.add_info  (type_, file.id);
            }

            self.add_action(&fm, file.id);
            self.add_status(&fm, file.id);
            self.add_fm    ( fm, file.id);
        }

        if is_empty {
            self.add_empty(file.id);
        }

        self.add_md(md, file.id);
    }


    // TODO: refactor and dedupe these add funcs
    fn add_type(&mut self, fm: &Mapping, id: FileId) -> Option<FmType> {
        let Ok(type_) = fm_get_property(fm, FmProperty::Type) else {
            None?
        };

        let Ok  (type_) = type_.as_str().try_into() else {
            warn!("unknown type prop: {type_}");
            None?
        };

        self.type_.insert(id, TypeComponent { type_ });

        Some(type_)
    }

    fn add_info(&mut self, type_: FmType, id: FileId) {
        if !matches!(type_, FmType::Info) {
            return;
        }

        self.info.insert(id);
    }

    fn add_action(&mut self, fm: &Mapping, id: FileId) {
        let Ok(action) = fm_get_property(fm, FmProperty::Action) else {
            return;
        };

        let Ok  (action) = action.as_str().try_into() else {
            warn!("unknown action prop: {action}");
            return;
        };

        self.action.insert(id, ActionComponent { action });
    }

    fn add_status(&mut self, fm: &Mapping, id: FileId) {
        let Ok(status) = fm_get_property(fm, FmProperty::Status) else {
            return;
        };

        let Ok  (status) = status.as_str().try_into() else {
            warn!("unknown status prop: {status}");
            return;
        };

        self.status.insert(id, StatusComponent { status });
    }

    fn add_empty(&mut self, id: FileId) {
        self.empty.insert(id);
    }

    fn add_fm(&mut self, fm: Mapping, id: FileId) {
        self.fm.insert(id, FmComponent { fm });
    }

    fn add_md(&mut self, md: String, id: FileId) {
        self.md_text.insert(id, MdTextComponent { text: md });
    }



}

pub fn parse_md_file(text: &str) -> Parsed {
    build_regex!(RE = RE_EMPTY);

    let is_empty = RE.is_match(&text);

    let blank = || {
        return Parsed {
            fm: None,
            md: text.to_string(),
            is_empty,
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
        md: text.to_string(),
        is_empty,
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


fn yaml_get_val_list_by_name(fm: &Mapping, name: &str) -> Result<Vec<String>, PropertyListError> {

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


pub fn is_unnamed(file_name: &str) -> bool {
    // (?i) - sets case insensitivity
    build_regex!(RE = r"(?i)^([\d \-_]*|Untitled(\s.*?)?)\.md$");

    RE.is_match(file_name)
}
