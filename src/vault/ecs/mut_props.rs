use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{Ecs, FileId, components::*}, fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}};

impl Ecs {

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
