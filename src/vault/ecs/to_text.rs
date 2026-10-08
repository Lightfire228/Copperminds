use std::fs;


use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{Ecs, EcsFileView, File, FileId, components::{EmptyFileComponent, FmComponent, FmProp}}, fm::{FmProperty, GetKey}};




impl Ecs {

    pub fn write_to_disk(&mut self, id: FileId) {

        if let Ok(fm) = self.get_component_mut::<FmComponent>(id) {
            fm.update();
        }

        let file = self.get(id).unwrap();
        let text = file.to_file_text();
        let file = file.file;

        // MAYBE: lock the file before writing to it?
        file.assert_unmodified();
        fs::write(&file.path, &text).unwrap();
    }
}

impl FmComponent {
    fn update(&mut self) {
        update_prop_str(&mut self.fm, FmProperty::Type,   &self.type_);
        update_prop_str(&mut self.fm, FmProperty::Action, &self.action);
        update_prop_str(&mut self.fm, FmProperty::Status, &self.status);
    }

}

fn update_prop_str<'a, T: GetKey>(fm: &mut Mapping, prop: FmProperty, value: &FmProp<T>) {
    if !value.modified {
        return;
    }

    let key = Value::String(prop.get_key());

    let Some(value) = &value.value else {
        fm.remove(key);
        return;
    };

    let value = Value::String(value.get_key());

    let slot = fm.entry(key).or_insert_with(|| Value::Null);

    *slot = value;
}


impl File {
    fn assert_unmodified(&self) {
        let text = fs::read_to_string(&self.path).unwrap();

        assert_eq!(self.raw_text, text, "there's a disturbance in the force");
    }
}



impl<'a> EcsFileView<'a> {

    pub fn to_file_text(&self) -> String {
        let md = self.get_md_text();

        let Some(FmComponent { fm, .. }) = self.fm else {
            return md;
        };




        format!("---\n{}---\n{}", fm_to_text(&fm), md)
    }


    fn get_md_text(&self) -> String {
        return self
            .md_text
            .map(|x| x.text.to_string())
            .unwrap_or_default()
        ;
    }
}

fn fm_to_text(fm: &Mapping) -> String {
    yaml_serde::to_string(fm).unwrap()
}


fn set_property(fm: &mut Mapping, property: String, value: Value) {
    let key = Value::String(property);

    fm.insert(key, value);
}

fn set_property_str(fm: &mut Mapping, property: String, value: String) {
    set_property(fm, property, Value::String(value));
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "todo"]
    fn test_unrelated_props() {
        todo!()
    }

}
