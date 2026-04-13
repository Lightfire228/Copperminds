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
    pub processing:  Option<Vec<String>>,
}

impl MdFile {

    pub fn new(file: &DirEntry) -> Self {
        parse_md_file(file)
    }

    pub fn assign_inbox(&mut self, inbox: String) {
        
        let mut fm = self.frontmatter.yaml.take().unwrap_or_else(|| Mapping::new());
        
        let key = Value::String("inbox".to_owned());
        let val = Value::String(inbox.clone());
        fm.insert(key, val);

        self.frontmatter.inbox = Some(inbox);
        self.frontmatter.yaml  = Some(fm);
    }

    pub fn assign_processing_tag(&mut self, tag: String) {
        
        
        let mut fm   = self.frontmatter.yaml      .take().unwrap_or_else(|| Mapping::new());
        let mut tags = self.frontmatter.processing.take().unwrap_or_else(|| Vec    ::new());

        tags.push(tag.clone());
        
        let key = Value::String("processing".to_owned());
        let val = Value::Sequence(Sequence::from_iter(tags
            .iter()
            .map (|t| Value::String(t.to_owned()))
        ));
        
        fm.insert(key, val);

        self.frontmatter.processing = Some(tags);
        self.frontmatter.yaml       = Some(fm);
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
}


fn parse_md_file(file: &DirEntry) -> MdFile {

    let name = || file.file_name().to_str().unwrap().to_owned();
    
    
    let blank = |text: String| {
        MdFile {
            frontmatter: Frontmatter {
                yaml:       None,
                inbox:      None,
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

    Frontmatter {
        yaml: Some(fm),
        
        processing,
        inbox,
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
