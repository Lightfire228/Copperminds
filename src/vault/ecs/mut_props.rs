use std::mem;

use yaml_serde::{Mapping, Value, to_value};

use crate::vault::{ecs::{Ecs, FileId, components::*}, fm::{FmAction, FmProject, FmProperty, FmStatus, FmType, GetKey}};

#[allow(dead_code)]
impl Ecs {

    pub fn replicate_changes_to_fm(&mut self, id: FileId) {

        let fm = self.get_component_or_insert::<FmYamlComponent, _>(id, || FmYamlComponent(FmProp::new_modified(Mapping::new())));
        let fm = fm.0.value.get_or_insert_with(|| Mapping::new());

        let mut fm = mem::take(fm);

        let fm_components = self
            .components
            .iter_mut()
            .filter_map(|x| x
                .1
                .get_component_mut(&id)
                .map(|x| x.fm_component_mut())
                .flatten()
            )
        ;

        for comp in fm_components {
            comp.replicate_changes(&mut fm);
        }

        self.set_component(id, FmYamlComponent(FmProp::new_modified(fm)));

    }

    pub fn set_info(&mut self, id: FileId) {
        self.remove_empty_flag(id);

        self.set_type(id, FmType::Info);
    }

    pub fn set_action(&mut self, id: FileId, action: FmAction) {
        self.remove_empty_flag(id);

        self.set_type     (id, FmType::Action);
        self.set_component(id, FmActionComponent(FmProp::new_modified(action)));
    }

    pub fn remove_action(&mut self, id: FileId) {
        _ = self
            .get_component_mut::<FmActionComponent>(id)
            .map(|x| x.0.clear())
        ;
    }

    pub fn set_status(&mut self, id: FileId, status: FmStatus) {
        self.remove_empty_flag(id);

        self.set_component(id, FmStatusComponent(FmProp::new_modified(status)));
    }

    pub fn remove_status(&mut self, id: FileId) {
        _ = self
            .get_component_mut::<FmStatusComponent>(id)
            .map(|x| x.0.clear())
        ;
    }

    pub fn set_project_name(&mut self, id: FileId, name: String) {
        self.remove_empty_flag(id);

        let is_workspace = self
            .get_component::<FmProjectComponent>(id)

            .map      (|x| x.0.value.as_ref().is_some_and(|x| x.is_workspace))
            .unwrap_or(false)
        ;

        self.set_project(id, FmProject{
            name,
            is_workspace,
        });

    }

    pub fn set_project(&mut self, id: FileId, project: FmProject) {
        self.remove_empty_flag(id);

        self.set_component(id, FmProjectComponent(FmProp::new_modified(project)));
    }

    #[allow(dead_code)]
    pub fn remove_project(&mut self, id: FileId) {
        _ = self
            .get_component_mut::<FmProjectComponent>(id)
            .map(|x| x.0.clear())
        ;
    }

    fn set_type(&mut self, id: FileId, type_: FmType) {
        self.remove_empty_flag(id);

        self.set_component(id, FmTypeComponent(FmProp::new_modified(type_)));
    }

    pub fn remove_type(&mut self, id: FileId)  {
        _ = self
            .get_component_mut::<FmTypeComponent>(id)
            .map(|x| x.0.clear())
        ;
    }


    fn remove_empty_flag(&mut self, id: FileId) {
        _ = self.remove_component::<EmptyFileComponent>(id);
    }

}




/// The main concern here is avoiding data loss.
/// Nothing should change the file in any way other than the intended effect
#[cfg(test)]
mod tests {
    use yaml_serde::Number;

    use crate::{test_utils::fm, vault::fm::{FmAction, FmStatus, FmType}};

    use super::*;


    fn get_fm(ecs: &Ecs, id: FileId) -> &Mapping {
        &ecs
            .get_component::<FmYamlComponent>(id)
            .expect("missing fm yaml component")
            .0
            .value
            .as_ref()
            .expect("missing fm yaml value")
    }

    #[test]
    fn test_mutations() {

        let mut ecs = Ecs::default();

        let id = fm!(ecs, );

        // Add props
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


        ecs.set_project_name(id, "proj".to_string());
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(
            FmProperty::Type    => FmType  ::Action,
            FmProperty::Status  => FmStatus::Archived,
            FmProperty::Action  => FmAction::Backlog,
            FmProperty::Project => "proj",
        ));

        ecs.set_project(id, FmProject {
            name:         "proj".to_string(),
            is_workspace: true,
        });
        ecs.replicate_changes_to_fm(id);

        assert_eq!(get_fm(&ecs, id), &fm!(
            FmProperty::Type    => FmType  ::Action,
            FmProperty::Status  => FmStatus::Archived,
            FmProperty::Action  => FmAction::Backlog,
            FmProperty::Project => fm!(
                "name"         => "proj",
                "is_workspace" => true,
            ),
        ));


        // Remove props
        ecs.remove_project         (id);
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

    #[test]
    fn test_name_collision() {
        use crate::vault::ecs::mut_props::*;

        let mut ecs = Ecs::default();

        let og_fm = fm!(
            "foo"         => true,
            "bar"         => "2000-01-01",
            "none"        => Value::Null,
            "lorem ipsum" => "dolor salut",

            // Make sure this stays intact
            "action" => fm!(
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

        let modified_fm = get_fm(&ecs, id);
        assert_eq!(&og_fm, modified_fm);
    }
}
