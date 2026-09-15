use yaml_serde::{Mapping, Value};

use crate::vault::{ecs::{ActionComponent, Ecs, FileId, FmComponent, StatusComponent, TypeComponent}, fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}};


impl Ecs {

    pub fn set_info(&mut self, id: FileId) {
        self.set_type(id, FmType::Info);

        self.info.insert(id);
        self.empty.remove(&id);
    }

    pub fn set_action(&mut self, id: FileId, action: FmAction) {
        let fm = self.set_type(id, FmType::Action);
        set_property(fm, FmProperty::Action.get_key(), action.get_key());

        self
            .action
            .entry         (id)
            .and_modify    (|a| a.action = action)
            .or_insert_with(|| ActionComponent {
                action,
            })
        ;
        self.empty.remove(&id);
    }

    pub fn set_status(&mut self, id: FileId, status: FmStatus) {

        let fm = self.get_fm_mut(id);
        set_property(fm, FmProperty::Status.get_key(), status.get_key());

        self
            .status
            .entry         (id)
            .and_modify    (|s| s.status = status)
            .or_insert_with(|| StatusComponent {
                status,
            })
        ;
        self.empty.remove(&id);
    }

    fn set_type(&mut self, id: FileId, type_: FmType) -> &mut Mapping {
        let fm = self.get_fm_mut(id);

        set_property(fm, FmProperty::Type.get_key(), type_.get_key());

        fm
    }

    fn get_fm_mut(&mut self, id: FileId) -> &mut Mapping {
        &mut self
            .fm
            .entry(id)
            .or_insert_with(|| FmComponent {
                fm: Mapping::new()
            })
            .fm
    }
}



fn set_property(fm: &mut Mapping, property: String, value: String) {

    let key = Value::String(property);
    let val = Value::String(value);

    fm.insert(key, val);
}
