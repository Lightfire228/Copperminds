use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{Ecs, FileId, components::*}, fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}};

impl Ecs {

    pub fn replicate_changes_to_fm(&mut self, id: FileId) {
        if let Ok(fm) = self.get_component_mut::<FmComponent>(id) {
            fm.update();
        }
    }

    pub fn set_info(&mut self, id: FileId) {
        self.set_type(id, FmType::Info);
    }

    pub fn set_action(&mut self, id: FileId, action: FmAction) {
        let fm = self.set_type(id, FmType::Action);

        fm.action = FmProp {
            value:    Some(action),
            modified: true,
        };
    }

    #[allow(dead_code)]
    pub fn remove_action(&mut self, id: FileId) {
        let fm = self.get_fm_mut(id);

        fm.action = FmProp {
            value:    None,
            modified: true,
        };
    }

    pub fn set_status(&mut self, id: FileId, status: FmStatus) {
        let fm = self.get_fm_mut(id);

        fm.status = FmProp {
            value:    Some(status),
            modified: true,
        };
    }

    #[allow(dead_code)]
    pub fn remove_status(&mut self, id: FileId) {
        let fm = self.get_fm_mut(id);

        fm.status = FmProp {
            value:    None,
            modified: true,
        };
    }

    fn set_type(&mut self, id: FileId, type_: FmType) -> &mut FmComponent {
        let fm = self.get_fm_mut(id);

        fm.type_ = FmProp {
            value:    Some(type_),
            modified: true,
        };

        fm
    }

    #[allow(dead_code)]
    pub fn remove_type(&mut self, id: FileId)  {
        let fm = self.get_fm_mut(id);

        fm.type_ = FmProp {
            value:    None,
            modified: true,
        };
    }


    fn get_fm_mut(&mut self, id: FileId) -> &mut FmComponent {
        self.remove_empty_flag(id);

        self.get_component_or_insert(id, || Default::default())
    }

    fn remove_empty_flag(&mut self, id: FileId) {
        _ = self.remove_component::<EmptyFileComponent>(id);
    }

}

impl FmComponent {
    fn update(&mut self) {
        update_prop_str(&mut self.fm, FmProperty::Type,   &mut self.type_);
        update_prop_str(&mut self.fm, FmProperty::Action, &mut self.action);
        update_prop_str(&mut self.fm, FmProperty::Status, &mut self.status);
    }

}

fn update_prop_str<'a, T: GetKey>(fm: &mut Mapping, prop: FmProperty, value: &mut FmProp<T>) {
    if !value.modified {
        return;
    }

    value.modified = false;

    let key = Value::String(prop.get_key());

    let Some(value) = &value.value else {
        fm.remove(key);
        return;
    };

    let value = Value::String(value.get_key());

    let slot = fm.entry(key).or_insert_with(|| Value::Null);

    *slot = value;
}





/// The main concern here is avoiding data loss.
/// Nothing should change the file in any way other than the intended effect
#[cfg(test)]
mod tests {
    use yaml_serde::Number;

    use crate::{test_utils::fm, vault::fm::{FmAction, FmStatus, FmType}};

    use super::*;


    fn get_fm(ecs: &Ecs, id: FileId) -> &Mapping {
        &ecs.get_component::<FmComponent>(id).unwrap().fm
    }

    #[test]
    fn test_mutations() {

        let mut ecs = Ecs::default();

        let id = fm!(ecs, );

        ecs.set_info               (id);
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(FmProperty::Type => FmType::Info));



        ecs.set_status             (id, FmStatus::Archived);
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(
            FmProperty::Type   => FmType  ::Info,
            FmProperty::Status => FmStatus::Archived,
        ));



        ecs.set_action             (id, FmAction::Backlog);
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(
            FmProperty::Type   => FmType  ::Action,
            FmProperty::Status => FmStatus::Archived,
            FmProperty::Action => FmAction::Backlog,
        ));



        ecs.remove_type            (id);
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(
            FmProperty::Status => FmStatus::Archived,
            FmProperty::Action => FmAction::Backlog,
        ));



        ecs.remove_action          (id);
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(
            FmProperty::Status => FmStatus::Archived,
        ));



        ecs.remove_status          (id);
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!());
    }

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
        ecs.set_info               (id);
        ecs.replicate_changes_to_fm(id);

        ecs.set_status             (id, FmStatus::Completed);
        ecs.replicate_changes_to_fm(id);

        ecs.set_action             (id, FmAction::Backlog);
        ecs.replicate_changes_to_fm(id);


        let modified_fm = get_fm(&ecs, id);

        for key in og_fm.keys() {
            let expected = og_fm      .get(key).unwrap();
            let modified = modified_fm.get(key).unwrap();

            assert_eq!(expected, modified);
        }

        // remove those changes
        ecs.remove_type            (id);
        ecs.replicate_changes_to_fm(id);

        ecs.remove_status          (id);
        ecs.replicate_changes_to_fm(id);

        ecs.remove_action          (id);
        ecs.replicate_changes_to_fm(id);


        let modified_fm = get_fm(&ecs, id);
        assert_eq!(&og_fm, modified_fm);
    }
}
