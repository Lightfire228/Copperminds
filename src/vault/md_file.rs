
use std::{fs, mem, path::Path};

use super::regex;
use walkdir::{DirEntry};
use yaml_serde::{Mapping, Sequence, Value};

const RE_EMPTY: &str = r"^\s*$";

#[derive(Debug)]
pub struct MdFile {
    pub entry:            DirEntry,
    pub frontmatter:      Option<Frontmatter>,
    pub file_name:        String,
    pub md_text:          String,
    pub raw_text:         String,
}

#[derive(Debug)]
pub struct Frontmatter {
    pub yaml:        Mapping,
    pub inbox:       Option<String>,
    pub status:      Option<String>,
    pub category:    Option<String>,

    // GTD
    pub type_:       Option<String>,
    pub context:     Option<String>,

    pub processing:  Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FmProperty {
    Inbox,
    Category,
    Status,
    Type,
    Context,
}

#[derive(Debug, Clone, Copy)]
pub enum FmType {
    Info,
    Action,
}
#[derive(Debug, Clone, Copy)]
pub enum FmContext {
    Todo,
    WaitingFor,
    Calendar,
    MaybeSomeday,
}

#[derive(Debug, Clone, Copy)]
pub enum FmPropertyList {
    Processing,
}

impl MdFile {

    pub fn new(file: &DirEntry) -> Self {
        parse_md_file(file)
    }

    pub fn refresh(&mut self) {
        let mut new = parse_md_file(&self.entry);
        mem::swap(&mut new, self);
    }

    pub fn write_file(&self) {
        let Some(fm) = self.frontmatter.as_ref().map(|x| &x.yaml) else {
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

    /// Does not check subdirectories
    pub fn is_in_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>
    {
        self
            .entry
            .path()
            .parent()
            .is_some_and(|f|
                f.ends_with(path)
            )
    }

    pub fn has_property(&self, property: FmProperty) -> bool {
        self
            .frontmatter
            .as_ref()
            .is_some_and(|f| f.get_property(property).is_some())
    }

    pub fn has_property_val(&self, property: FmProperty, value: &str) -> bool {
        self
            .frontmatter
            .as_ref()
            .is_some_and(|f| f.get_property(property) == Some(value))
    }

    // TODO: these should probably return a "file state" enum
    // FileState::Modified
    pub fn has_property_val_any(&self, property: FmProperty, values: &[&str]) -> bool {
        self.frontmatter
            .as_ref     ()
            .and_then   (|f| f.get_property(property))
            .is_some_and(|p| values.contains(&p) )
    }

    pub fn is_uncategorized(&self) -> bool {
        self
            .frontmatter
            .as_ref    ()
            .is_none_or(|fm| fm.category.is_none())
    }

    /// GTD Type
    pub fn is_untyped(&self) -> bool {
           !self.has_property(FmProperty::Type)
        && !self.entry
            .path     ()
            .ancestors()
            .any      (|p| p.ends_with("03 Data"))
    }

    /// GTD Context
    pub fn is_uncontextualized(&self) -> bool {
              !self.has_property    (FmProperty::Context)
           &&  self.has_property_val(FmProperty::Type, "action")

    }





    pub fn assign_property(&mut self, property: FmProperty, value: String) {

        let fm = self.frontmatter.get_or_insert_with(Frontmatter::blank);

        let key = Value::String(property.get_key());
        let val = Value::String(value   .clone());
        fm.yaml.insert(key, val);

        fm.set_property(property, value);
    }

    pub fn remove_property(&mut self, property: FmProperty) {

        let Some(fm) = &mut self.frontmatter else {
            return;
        };

        fm.yaml.remove(property.get_key());
        _ = fm.take_property(property);
    }

    pub fn remove_property_list(&mut self, property: FmPropertyList) -> bool {

        let Some(fm) = &mut self.frontmatter else {
            return false;
        };

        fm.yaml.remove(property.get_key());
        _ = fm.take_property_list(property);

        true
    }

    pub fn push_list_val(&mut self, list: FmPropertyList, tag: String) {

        let     fm   = self.frontmatter.get_or_insert_with(Frontmatter::blank);
        let mut tags = fm.take_property_list(list);

        tags.push(tag.clone());

        let key = Value::String  (list.get_key());
        let val = Value::Sequence(Sequence::from_iter(tags
            .iter()
            .map (|t| Value::String(t.to_owned()))
        ));

        fm.yaml.insert(key, val);
        fm.set_property_list(list, tags);
    }

    pub fn rename_property(&mut self, old_prop: FmProperty, new_prop: FmProperty) -> bool {

        let Some(fm) = &mut self.frontmatter else {
            return false;
        };

        if let Some(existing) = fm.get_property(new_prop) {
            panic!("{}: {} is already defined for file: {}", new_prop.get_key(), existing, self.file_name);
        }

        let Some(old_val) = fm.take_property(old_prop) else {
            return false;
        };

        fm.yaml.remove(old_prop.get_key());

        self.assign_property(new_prop, old_val);

        true

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
            FmProperty::Inbox    => self.inbox   .as_ref().map(|p| p.as_str()),
            FmProperty::Category => self.category.as_ref().map(|p| p.as_str()),
            FmProperty::Status   => self.status  .as_ref().map(|p| p.as_str()),
            FmProperty::Type     => self.type_   .as_ref().map(|p| p.as_str()),
            FmProperty::Context  => self.status  .as_ref().map(|p| p.as_str()),
        }
    }

    fn set_property(&mut self, property: FmProperty, value: String) {
        match property {
            FmProperty::Inbox    => self.inbox    = Some(value),
            FmProperty::Category => self.category = Some(value),
            FmProperty::Status   => self.status   = Some(value),
            FmProperty::Type     => self.type_    = Some(value),
            FmProperty::Context  => self.context  = Some(value),
        }
    }

    fn take_property(&mut self, property: FmProperty) -> Option<String> {
        match property {
            FmProperty::Inbox    => self.inbox    .take(),
            FmProperty::Category => self.category .take(),
            FmProperty::Status   => self.status   .take(),
            FmProperty::Type     => self.type_    .take(),
            FmProperty::Context  => self.context  .take(),
        }
    }

    fn set_property_list(&mut self, list: FmPropertyList, value: Vec<String>) {
        match list {
            FmPropertyList::Processing => self.processing = Some(value)
        }
    }

    fn take_property_list(&mut self, list: FmPropertyList) -> Vec<String> {
        match list {
            FmPropertyList::Processing => self.processing.take().unwrap_or_else(|| Vec::new())
        }
    }

}

impl Frontmatter {
    pub fn blank() -> Self {
        Self {
            yaml:       Mapping::new(),
            inbox:      None,
            status:     None,
            category:   None,
            processing: None,

            type_:      None,
            context:    None,
        }
    }
}

impl FmProperty {
    pub fn get_key(&self) -> String {
        match &self {
            FmProperty::Inbox    => "inbox"   .to_owned(),
            FmProperty::Category => "category".to_owned(),
            FmProperty::Status   => "status"  .to_owned(),
            FmProperty::Type     => "type"    .to_owned(),
            FmProperty::Context  => "context" .to_owned(),
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
            frontmatter: None,

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
        frontmatter: Some(parse_frontmatter(fm)),
        entry:       file.clone(),
        file_name:   name(),
        md_text:     parsed.body,
        raw_text:    parsed.raw_text,
    }
}

fn parse_frontmatter(fm: Mapping) -> Frontmatter {

    macro_rules! get_str {
        ($val:literal) => {
            fm
                .get     ($val)
                .and_then(|i| i.as_str())
                .map     (|i| i.to_owned())

        };
    }

    let processing = get_processing_tag(&fm);
    let inbox = get_str!("inbox");

    let status   = get_str!("status");
    let category = get_str!("category");
    let type_    = get_str!("type");
    let context  = get_str!("context");

    Frontmatter {
        yaml: fm,

        processing,
        inbox,
        status,
        category,

        type_,
        context,
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
