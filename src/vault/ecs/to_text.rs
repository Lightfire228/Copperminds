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



#[cfg(test)]
mod tests {
    use yaml_serde::Number;

    use crate::{test_utils::fm, vault::fm::{FmAction, FmStatus}};

    use super::*;

    #[test]
    fn test_unrelated_props() {

        use crate::vault::ecs::mut_props::*;

        let mut ecs = Ecs::default();

        let og_fm = fm!(
            "foo"         => true,
            "bar"         => "2000-01-01",
            "none"        => Value::Null,
            "lorem ipsum" => "dolor salut",

            "baz" => fm!(
                "nested" => true,
                "ooga"   => 10,
                "booga"  => 10.0,
            ),
        );

        let id = ecs.load_fm_test(og_fm.clone());

        // make a bunch of changes
        ecs.set_info         (id);
        ecs.replicate_changes(id);

        ecs.set_status       (id, FmStatus::Completed);
        ecs.replicate_changes(id);

        ecs.set_action       (id, FmAction::Backlog);
        ecs.replicate_changes(id);


        let modified_fm = &ecs.get_component::<FmComponent>(id).unwrap().fm;

        for key in og_fm.keys() {
            let expected = og_fm      .get(key).unwrap();
            let modified = modified_fm.get(key).unwrap();

            assert_eq!(expected, modified);
        }

        // remove those changes
        ecs.remove_type      (id);
        ecs.replicate_changes(id);

        ecs.remove_status    (id);
        ecs.replicate_changes(id);

        ecs.remove_action    (id);
        ecs.replicate_changes(id);


        let modified_fm = &ecs.get_component::<FmComponent>(id).unwrap().fm;
        assert_eq!(&og_fm, modified_fm);

    }

    impl Ecs {
        fn replicate_changes(&mut self, id: FileId) {
            self.get_component_mut::<FmComponent>(id).unwrap().update();
        }
    }
}
