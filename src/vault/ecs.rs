mod delete_empty_unnamed;
mod parse;
mod mut_props;
mod to_text;

pub mod components;

use std::{any::{Any, TypeId}, collections::HashMap, path::{Path, PathBuf}};

use yaml_serde::Mapping;


type FileId = file_id::FileId;

use crate::{config::Config, vault::{ecs::components::*, fm::{FmAction, FmStatus, FmType}}};

use super::build_regex;

static CAST_ERR: &str = "the types done got all fucked up";

#[derive(Default)]
pub struct Ecs {

    file:       HashMap<FileId, File>,

    components: HashMap<TypeId, Box<dyn ComponentList>>,
}

#[derive(Debug)]
pub struct File {
    pub name:     String,
    pub unnamed:  bool,

    /// is the absolute file path to the file
    pub path:     PathBuf,

    /// this is for debugging purposes
    /// mainly to confirm there haven't been any untracked changes since the last vault scan
    pub raw_text: String,
}


pub struct NewFile {
    pub id:       FileId,

    pub path:     PathBuf,
    pub raw_text: String,
    pub name:     String,
}

trait ComponentList: Any + Send + 'static {
    fn remove(&mut self, id: &FileId);
}

impl<'a, T> ComponentList for HashMap<FileId, T>
where
    T: Component + Any,
{
    fn remove(&mut self, id: &FileId) {
        self.remove(id);
    }
}

type ComponentMap<T> = HashMap<FileId, T>;



impl Ecs {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_component<T: Component>(&mut self, comp_id: FileId, value: T) {
        let list = self.get_comp_list_or_insert();

        list.insert(comp_id, value);
    }

    // --- queries

    pub fn get_all(&self) -> impl Iterator<Item = EcsFileView<'_>> {
        self.file.iter().map(|x| self.to_file_view(*x.0, x.1))
    }

    pub fn get(&self, id: FileId) -> Option<EcsFileView<'_>> {
        self.file.get(&id).map(|x| self.to_file_view(id, x))
    }

    pub fn iter_components<T: Component>(&self) -> impl Iterator<Item = (&FileId, &T)> {
        self.get_comp_list().unwrap().iter()
    }

    pub fn iter_components_mut<T: Component>(&mut self) -> impl Iterator<Item = (&FileId, &mut T)> {
        self.get_comp_list_mut().unwrap().iter_mut()
    }


    pub fn get_component<T: Component>(&self, comp_id: FileId) -> Result<&T, ComponentError> {
        type Er = ComponentError;

        let list = self
            .get_comp_list()
            .ok_or(Er::NoComponentsOfThatType)?;

        Ok(list
            .get(&comp_id)
            .ok_or(Er::ComponentNotFound)?
        )
    }

    pub fn get_component_mut<T: Component>(&mut self, comp_id: FileId) -> Result<&mut T, ComponentError> {
        type Er = ComponentError;

        let list = self
            .get_comp_list_mut()
            .ok_or(Er::NoComponentsOfThatType)?;

        Ok(list
            .get_mut(&comp_id)
            .ok_or(Er::ComponentNotFound)?
        )
    }

    pub fn get_component_or_insert<T, F>(&mut self, comp_id: FileId, func: F) -> &mut T
    where
        T: Component,
        F: Fn() -> T
    {

        self
            .get_comp_list_or_insert()

            .entry         (comp_id)
            .or_insert_with(func)

    }

    pub fn remove_component<T: Component>(&mut self, comp_id: FileId) -> Result<T, ComponentError> {
        let list = self.get_comp_list_mut().ok_or(ComponentError::NoComponentsOfThatType)?;

        list.remove(&comp_id).ok_or(ComponentError::ComponentNotFound)


    }

    fn to_file_view<'a>(&'a self, id: FileId, file: &'a File) -> EcsFileView<'a> {
        EcsFileView {
            id,
            file,
            fm:       self.get_component(id).ok(),
            md_text:  self.get_component(id).ok(),
            empty:    self.get_component(id).ok(),
            type_:    self.get_component(id).ok(),
            info:     self.get_component(id).ok(),
            action:   self.get_component(id).ok(),
            status:   self.get_component(id).ok(),
            project:  self.get_component(id).ok(),
        }
    }

    pub fn get_component_counts<'a, T: Component>(&'a self) -> usize {
        self.get_comp_list::<T>().map_or_else(|| 0, |x| x.len())
    }

    // --- writes

    /// Removes file from all components.
    /// NOTE: does not delete file from disk
    pub fn remove_file(&mut self, id: FileId) -> File {
        self
            .components
            .iter_mut()
            .for_each(|x| {
                x.1.remove(&id);
            })
        ;

        self.file.remove(&id).unwrap()
    }


    // --- utils

    fn get_comp_list<T: Component>(&self) -> Option<&ComponentMap<T>> {
        let type_id = map_id::<T>();

        let list = self.components.get(&type_id)?.as_ref() as &dyn Any;

        Some(list
            .downcast_ref()
            .expect(CAST_ERR)
        )
    }

    fn get_comp_list_mut<T: Component>(&mut self) -> Option<&mut ComponentMap<T>> {
        let type_id = map_id::<T>();

        let list = self.components.get_mut(&type_id)?.as_mut() as &mut dyn Any;

        Some(list
            .downcast_mut()
            .expect(CAST_ERR)
        )
    }

    fn get_comp_list_or_insert<T: Component>(&mut self) -> &mut HashMap<FileId, T> {
        let type_id = map_id::<T>();

        let list = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(HashMap::<FileId, T>::new()))
            .as_mut()
        ;


        (list as &mut dyn Any)
            .downcast_mut::<HashMap<FileId, T>>()
            .expect(CAST_ERR)
    }

}



impl<'a> EcsFileView<'a> {
    pub fn status_eq(&'a self, status: FmStatus) -> bool {
        self.status.is_some_and(|s| s.status == status)
    }
    pub fn action_eq(&'a self, action: FmAction) -> bool {
        self.action.is_some_and(|a| a.action == action)
    }
    pub fn type_eq(&'a self, type_: FmType) -> bool {
        self.type_.is_some_and(|t| t.type_ == type_)
    }

    // TODO: remove the type prop and just infer the type from it's top level props
    // - info
    // - action
    /// Does not account for status prop
    pub fn is_actionable(&'a self) -> bool {
        self.type_eq(FmType::Action) && self.action.is_some()
    }

    pub fn is_open(&'a self) -> bool {
        self.status.is_none_or(|s| {
               s.status != FmStatus::Archived
            && s.status != FmStatus::Completed
        })
    }

    pub fn is_info(&'a self) -> bool {
        self.info.is_some()
    }

    pub fn is_empty(&'a self) -> bool {
        self.empty.is_some()
    }

    pub fn needs_type(&'a self) -> bool {
        self.type_.is_none()
    }

    pub fn needs_sorting(&'a self) -> bool {
        [
            self.needs_type(),
            self.needs_action_assigned(),
            self.is_unnamed(),
        ]
            .iter()
            .any(|f| *f)
    }

    pub fn is_unnamed(&'a self) -> bool {
        // (?i) - sets case insensitivity
        build_regex!(RE = r"(?i)^([\d \-_]*|Untitled([\s\d\-_\(\)]*?)?)\.md$");

        RE.is_match(&self.file.name)
    }

    pub fn needs_action_assigned(&'a self) -> bool {
        self.type_eq(FmType::Action) && self.action.is_none()
    }

    pub fn is_archived(&'a self) -> bool {
        self.status_eq(FmStatus::Archived)
    }

    pub fn is_completed(&'a self) -> bool {
        self.status_eq(FmStatus::Completed)
    }

    pub fn to_file_text(&self) -> String {
        let md = self.get_md_text();

        let Some(FmComponent { fm }) = self.fm else {
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

#[inline(always)]
fn map_id<T: Component>() -> TypeId {
    TypeId::of::<HashMap<FileId, T>>()
}


fn fm_to_text(fm: &Mapping) -> String {
    yaml_serde::to_string(fm).unwrap()
}


// impl ComponentKind {
//     fn all() -> Vec<Self> {
//         all::<ComponentKind>().collect()
//     }
// }

pub enum ComponentError {
    NoComponentsOfThatType,
    ComponentNotFound,
}


#[cfg(test)]
mod tests {

    use yaml_serde::{Mapping, Value};

    use crate::{test_utils::{self, id, mapping_to_str}, vault::{fm::{FmAction, FmProperty, FmStatus, FmType, GetKey}}};

    use super::*;

    macro_rules! fm {
        ($ecs:ident, $($key:expr => $value:expr),*$(,)? ) => {{
            #[allow(unused_mut)] // reason: ignore warning for empty fm creation
            let mut fm = Mapping::new();

            $(
                fm.insert(Value::String($key.get_key()), Value::String($value.get_key()));
            )*

            let id = id();

            $ecs.new_file(NewFile {
                id,
                path:     PathBuf::new(),
                raw_text: mapping_to_str(fm),
                name:     String::new(),
            });

            id
        }};
    }

    #[test]
    fn test_type_sorting() {

        let mut ecs = Ecs::default();

        let untyped = fm!(ecs, );
        let info    = fm!(ecs, FmProperty::Type => FmType::Info);
        let action  = fm!(ecs, FmProperty::Type => FmType::Action);


        let untyped = ecs.get(untyped).unwrap();
        let info    = ecs.get(info)   .unwrap();
        let action  = ecs.get(action) .unwrap();

        assert_eq!(untyped.needs_type(), true , "untyped");
        assert_eq!(info   .needs_type(), false, "info");
        assert_eq!(action .needs_type(), false, "action");

        assert!(info  .is_info());
        assert!(info  .type_eq(FmType::Info));
        assert!(action.type_eq(FmType::Action));
    }

    #[test]
    fn test_action_sorting() {

        let mut ecs = Ecs::default();

        let no_action_info     = fm!(ecs, FmProperty::Type => FmType::Info);
        let no_action          = fm!(ecs,                                     FmProperty::Action => FmAction::Todo);
        let needs_action       = fm!(ecs, FmProperty::Type => FmType::Action);
        let action_todo        = fm!(ecs, FmProperty::Type => FmType::Action, FmProperty::Action => FmAction::Todo);
        let action_waiting_for = fm!(ecs, FmProperty::Type => FmType::Action, FmProperty::Action => FmAction::WaitingFor);


        let no_action_info     = ecs.get(no_action_info)    .unwrap();
        let no_action          = ecs.get(no_action)         .unwrap();
        let needs_action       = ecs.get(needs_action)      .unwrap();
        let action_todo        = ecs.get(action_todo)       .unwrap();
        let action_waiting_for = ecs.get(action_waiting_for).unwrap();

        assert_eq!(no_action_info    .needs_action_assigned(), false, "no_action_info     needs_action_assigned");
        assert_eq!(no_action         .needs_action_assigned(), false, "no_action          needs_action_assigned");
        assert_eq!(needs_action      .needs_action_assigned(), true,  "needs_action       needs_action_assigned");
        assert_eq!(action_todo       .needs_action_assigned(), false, "action_todo        needs_action_assigned");
        assert_eq!(action_waiting_for.needs_action_assigned(), false, "action_waiting_for needs_action_assigned");

        assert_eq!(no_action_info    .is_actionable(),         false, "no_action_info     is_actionable");
        assert_eq!(no_action         .is_actionable(),         false, "no_action          is_actionable");
        assert_eq!(needs_action      .is_actionable(),         false, "needs_action       is_actionable");
        assert_eq!(action_todo       .is_actionable(),         true,  "action_todo        is_actionable");
        assert_eq!(action_waiting_for.is_actionable(),         true,  "action_waiting_for is_actionable");
    }

    #[test]
    fn test_status_sorting() {
        let mut ecs = Ecs::default();

        let archive   = fm!(ecs, FmProperty::Status => "archive");
        let archived  = fm!(ecs, FmProperty::Status => FmStatus::Archived);
        let complete  = fm!(ecs, FmProperty::Status => "complete");
        let completed = fm!(ecs, FmProperty::Status => FmStatus::Completed);

        let archive   = ecs.get(archive  ).unwrap();
        let archived  = ecs.get(archived ).unwrap();
        let complete  = ecs.get(complete ).unwrap();
        let completed = ecs.get(completed).unwrap();

        assert_eq!(archive  .is_archived(),  true,  "archive   is_archived");
        assert_eq!(archived .is_archived(),  true,  "archived  is_archived");
        assert_eq!(complete .is_archived(),  false, "complete  is_archived");
        assert_eq!(completed.is_archived(),  false, "completed is_archived");

        assert_eq!(archive  .is_completed(), false, "archive   is_completed");
        assert_eq!(archived .is_completed(), false, "archived  is_completed");
        assert_eq!(complete .is_completed(), true,  "complete  is_completed");
        assert_eq!(completed.is_completed(), true,  "completed is_completed");
    }

    #[test]
    #[ignore = "TODO"]
    fn test_project_sorting() {
        todo!()
    }
}
