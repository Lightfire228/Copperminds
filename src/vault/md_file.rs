use std::{collections::HashMap, env, fs, mem, ops::Deref, path::PathBuf, sync::LazyLock};

use super::regex;
use serde::de;
use walkdir::{DirEntry, WalkDir};
use yaml_serde::{Mapping, Sequence, Value};
use trash;

const RE_EMPTY: &str = r"^\s*$";

pub struct MdFile {
    pub entry:            DirEntry,
    pub frontmatter:      Frontmatter,
    pub file_name:        String,
    pub md_text:          String,
    pub raw_text:         String,
}

pub struct Frontmatter {
    pub yaml:        Option<Mapping>,
    pub inbox:       Option<String>,
    pub status:      Option<String>,
    pub processing:  Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FmProperty {
    Inbox,
    Status,
}

#[derive(Debug, Clone, Copy)]
pub enum FmPropertyList {
    Processing,
}

impl MdFile {

    pub fn new(file: &DirEntry) -> Self {
        parse_md_file(file)
    }

    pub fn write_file(&self) {
        let Some(fm) = &self.frontmatter.yaml else {
            fs::write(self.entry.path(), &self.md_text).unwrap();
            return;
        };

        let fm_text = yaml_serde::to_string(fm).unwrap();
        let text    = format!("---\n{fm_text}---\n{}", self.md_text);

        fs::write(self.entry.path(), &text).unwrap();
    }


    pub fn is_empty(&self) -> bool {
        regex!(RE = RE_EMPTY);

        RE.is_match(&self.raw_text)
    }

    pub fn is_md_empty(&self) -> bool {
        regex!(RE = RE_EMPTY);

        RE.is_match(&self.md_text)
    }

    pub fn is_unnamed(&self) -> bool {
        regex!(RE = r"^([\d \-_]*|Untitled.*?)\.md$");

        RE.is_match(&self.file_name)
    }

    pub fn has_property_val(&self, property: FmProperty, value: &str) -> bool {
        self.frontmatter.get_property(property) == Some(value)
    }

    pub fn has_property_val_any(&self, property: FmProperty, values: &[&str]) -> bool {
        let Some(prop) = self.frontmatter.get_property(property) else {
            return false;
        };


        values.contains(&prop)
    }


    pub fn assign_inbox(&mut self, inbox: String) {
        self.assign_property(FmProperty::Inbox, inbox);
    }
    
    pub fn assign_processing_tag(&mut self, tag: String) {
        self.push_property_list(FmPropertyList::Processing, tag);
    }

    pub fn assign_property(&mut self, property: FmProperty, value: String) {
        
        let mut fm = self.frontmatter.yaml.take().unwrap_or_else(|| Mapping::new());
        
        let key = Value::String(property.get_key());
        let val = Value::String(value   .clone());
        fm.insert(key, val);

        self.frontmatter.yaml = Some(fm);
        self.frontmatter.set_property(property, value);
    }

    pub fn push_property_list(&mut self, list: FmPropertyList, tag: String) {
        
        let mut fm   = self.frontmatter.yaml.take().unwrap_or_else(|| Mapping::new());
        let mut tags = self.frontmatter.take_property_list_mut(list);

        tags.push(tag.clone());
        
        let key = Value::String  (list.get_key());
        let val = Value::Sequence(Sequence::from_iter(tags
            .iter()
            .map (|t| Value::String(t.to_owned()))
        ));
        
        fm.insert(key, val);

        self.frontmatter.yaml = Some(fm);
        self.frontmatter.set_property_list(list, tags);
    }
}

impl PartialEq for MdFile {
    fn eq(&self, other: &Self) -> bool {
        self.entry.path() == other.entry.path()
    }
}

impl Eq for MdFile {}

impl Frontmatter {
    pub fn new(fm: Mapping) -> Self {
        parse_frontmatter(fm)
    }

    fn get_property(&self, property: FmProperty) -> Option<&str> {

        match property {
            FmProperty::Inbox  => self.inbox .as_ref().map(|p| p.as_str()),
            FmProperty::Status => self.status.as_ref().map(|p| p.as_str()),
        }
    }

    fn set_property(&mut self, property: FmProperty, value: String) {

        match property {
            FmProperty::Inbox  => self.inbox  = Some(value),
            FmProperty::Status => self.status = Some(value),
        }
    }

    fn set_property_list(&mut self, list: FmPropertyList, value: Vec<String>) {
        match list {
            FmPropertyList::Processing => self.processing = Some(value)
        }
    }

    fn take_property_list_mut(&mut self, list: FmPropertyList) -> Vec<String> {
        match list {
            FmPropertyList::Processing => self.processing.take().unwrap_or_else(|| Vec::new())
        }
    }

}

impl FmProperty {
    pub fn get_key(&self) -> String {
        match &self {
            FmProperty::Inbox  => "inbox" .to_owned(),
            FmProperty::Status => "status".to_owned()
        }
    }
}

impl FmPropertyList {
    pub fn get_key(&self) -> String {
        match &self {
            FmPropertyList::Processing => "processing".to_owned(),
        }
    }
}


fn parse_md_file(file: &DirEntry) -> MdFile {

    let name = || file.file_name().to_str().unwrap().to_owned();
    
    
    let blank = |text: String| {
        MdFile {
            frontmatter: Frontmatter {
                yaml:       None,
                inbox:      None,
                status:     None,
                processing: None,
            },
            
            entry:     file.clone(),
            file_name: name(),
            md_text:   text.clone(),
            raw_text:  text,
        }
    };

    let parsed = get_frontmatter_text(&file);

    let parsed = match parsed {
        Parsed::None       (text)      => return blank(text),
        Parsed::Frontmatter(parsed_fm) => parsed_fm
    };

    let Some(fm)   = yaml_serde::from_str::<Mapping>(&parsed.fm).ok() else {
        return blank(parsed.raw_text);
    };

    MdFile {
        frontmatter: parse_frontmatter(fm),
        entry:       file.clone(),
        file_name:   name(),
        md_text:     parsed.body,
        raw_text:    parsed.raw_text,
    }
}

fn parse_frontmatter(fm: Mapping) -> Frontmatter {
    let processing = get_processing_tag(&fm);
    let inbox      = fm
        .get     ("inbox")
        .and_then(|i| i.as_str())
        .map     (|i| i.to_owned())
    ;

    let status     = fm
        .get     ("status")
        .and_then(|i| i.as_str())
        .map     (|i| i.to_owned())
    ;

    Frontmatter {
        yaml: Some(fm),
        
        processing,
        inbox,
        status,
    }
        
}

fn get_processing_tag(fm: &Mapping) -> Option<Vec<String>> {

    let processing = fm.get("processing")?;

    if processing.is_mapping() {
        panic!("'processing' tag is a mapping, expected list or single value");
    }

    let values = if processing.is_sequence() {
        processing.as_sequence().unwrap().as_slice()
    } else {
        &[processing.to_owned()]
    };

    let values: Vec<_> = values
        .iter()
        .map (|v| match v {
            Value::Bool  (b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_owned(),

            _  => panic!("expect value to be a bool, string, or number"),
        })
        .collect()
    ;

    Some(values)
}


fn get_frontmatter_text(file: &DirEntry) -> Parsed {

    // (?ms) set flags
    // m     multi-line mode: ^ and $ match begin/end of line
    // s     allow . to match \n
    regex!(RE = r"^(?ms)---\s*(.+?)^---\s*$(.*)");

    let text = fs::read_to_string(file.path()).unwrap();

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
