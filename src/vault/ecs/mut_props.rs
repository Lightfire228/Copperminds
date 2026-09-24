use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{ActionComponent, Ecs, FileId, components::*}, fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}};

// since file formatting only uses the FmComponent, it is necessary to keep it in sync
// when modifying anything frontmatter related
impl Ecs {

    pub fn set_info(&mut self, id: FileId) {
        self.set_type(id, FmType::Info);

        self.add_component(id, InfoComponent);
        self.remove_empty (id);
    }

    pub fn set_action(&mut self, id: FileId, action: FmAction) {
        let fm = self.set_type(id, FmType::Action);
        set_property(fm, FmProperty::Action.get_key(), action.get_key());

        self.get_component_or_insert(id, || ActionComponent {
            action,
        });
        self.remove_empty (id);
    }

    pub fn set_status(&mut self, id: FileId, status: FmStatus) {

        let fm = self.get_fm_mut(id);
        set_property(fm, FmProperty::Status.get_key(), status.get_key());

        self.get_component_or_insert(id, || StatusComponent {
            status,
        });
        self.remove_empty(id);
    }


    fn set_type(&mut self, id: FileId, type_: FmType) -> &mut Mapping {
        let fm = self.get_fm_mut(id);

        set_property(fm, FmProperty::Type.get_key(), type_.get_key());

        fm
    }

    fn get_fm_mut(&mut self, id: FileId) -> &mut Mapping {
        self.remove_empty(id);

        &mut self.get_component_or_insert(id, || FmComponent {
            fm: Mapping::new()
        })
            .fm
    }

    fn remove_empty(&mut self, id: FileId) {
        _ = self.remove_component::<EmptyComponent>(id);
    }
}



fn set_property(fm: &mut Mapping, property: String, value: String) {

    let key = Value::String(property);
    let val = Value::String(value);

    fm.insert(key, val);
}




/// The main concern here is avoiding data loss.
/// Nothing should change the file in any way other than the intended effect
#[cfg(test)]
mod tests {
    use std::{vec};

    use crate::test_utils::load_file;


    fn _load_test_bodies() -> Vec<String> {
        vec![
            load_file("parsing/test_body_01.md"),
            load_file("parsing/test_body_02.md"),
        ]
    }

    #[test]
    #[ignore = "todo"]
    fn test_property_writes() {
        // macro_rules! value {
        //     ($x:literal) => {
        //         Value::String($x.to_owned())
        //     };
        // }

        // let bodies = load_test_bodies();

        // for text in bodies {
        //     let mut body = parse_md_file(&text);
        //     let mut fm   = Mapping::new();

        //     macro_rules! assert {
        //         () => {
        //             assert_eq!(body.fm.as_ref(), Some(&fm));
        //             assert_eq!(body.md,          text);
        //         };
        //     }

        //     macro_rules! add {
        //         ($key:literal, $val:literal) => {
        //             body.set_property($key.to_owned(), $val.to_owned());
        //             fm.insert(value!($key), value!($val));

        //             assert!();
        //         };
        //     }

        //     add!("test prop 01", "test val 01");
        //     add!("test prop 02", "test val 02");
        //     add!("test prop 03", "test val 03");


        //     // test modify
        //     body.set_property("test prop 02".to_owned(), "changed".to_owned());

        //     let x = fm.get_mut(value!("test prop 02")).unwrap();
        //     *x = value!("changed");

        //     assert!();


        //     // test delete
        //     body.remove_property("test prop 02".to_owned());
        //     fm.remove(value!("test prop 02"));

        //     assert!();
        // }
    }

}
