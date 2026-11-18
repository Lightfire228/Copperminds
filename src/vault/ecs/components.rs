use yaml_serde::Mapping;

use crate::{config::Config, vault::{ecs::{File, FileId}, fm::*}};

pub trait Component: Send + 'static {
    fn fm_component    (&    self) -> Option<&    dyn FmComponent> { None }
    fn fm_component_mut(&mut self) -> Option<&mut dyn FmComponent> { None }
}

pub trait FmComponent {
    fn replicate_changes(&mut self, fm: &mut Mapping);

    // fn set_modified(&mut self, value: bool);
}


#[derive(Debug)]
pub struct FmProp<T> {
    pub value:    Option<T>,
    pub modified: bool,
}


#[derive(Debug)] pub struct FmYamlComponent   (pub FmProp<Mapping>);
#[derive(Debug)] pub struct FmTypeComponent   (pub FmProp<FmType>);
#[derive(Debug)] pub struct FmActionComponent (pub FmProp<FmAction>);
#[derive(Debug)] pub struct FmStatusComponent (pub FmProp<FmStatus>);
#[derive(Debug)] pub struct FmProjectComponent(pub FmProp<FmProject>);

#[derive(Debug)]
pub struct MdTextComponent {
    pub text: String,
}

/// Tracks if the file, before parsing frontmatter, has any non-whitespace character in it
#[derive(Debug)]
pub struct EmptyFileComponent;

/// Tracks if the file has a name that's illegal on Android
#[derive(Debug)]
pub struct IllegalNameComponent;


// impl<T> Default for FmProp<T> {
//     fn default() -> Self {
//         Self {
//             value:    None,
//             modified: false,
//         }
//     }
// }

impl<T> FmProp<T> {
    pub fn new_modified(value: T) -> Self {
        Self {
            value:    Some(value),
            modified: true,
        }
    }

    pub fn new_unmodified(value: T) -> Self {
        Self {
            value:    Some(value),
            modified: false,
        }
    }

    pub fn new_empty(&mut self) -> Self {
        Self {
            value:    None,
            modified: true,
        }
    }

    pub fn clear(&mut self) {
        self.value = None;
        self.modified = true;
    }
}

macro_rules! impl_component_with_fm {
    ($ty:ty) => {
        impl Component for $ty {
            fn fm_component(&self) -> Option<&dyn FmComponent> {
                Some(self as &dyn FmComponent)
            }

            fn fm_component_mut(&mut self) -> Option<&mut dyn FmComponent> {
                Some(self as &mut dyn FmComponent)
            }
        }

    };
}

macro_rules! impl_component {
    ($ty:ty) => {
        impl Component for $ty {
            fn fm_component    (&    self) -> Option<&    dyn FmComponent> { None }

            fn fm_component_mut(&mut self) -> Option<&mut dyn FmComponent> { None }
        }
    };
}

impl_component_with_fm!(FmTypeComponent);
impl_component_with_fm!(FmActionComponent);
impl_component_with_fm!(FmStatusComponent);
impl_component_with_fm!(FmProjectComponent);

impl_component!(FmYamlComponent);
impl_component!(MdTextComponent);
impl_component!(EmptyFileComponent);
impl_component!(IllegalNameComponent);




impl FmComponent for FmTypeComponent {
    fn replicate_changes(&mut self, fm: &mut Mapping) {
        update_prop_str(fm, FmProperty::Type, &mut self.0);
    }
}

impl FmComponent for FmActionComponent {
    fn replicate_changes(&mut self, fm: &mut Mapping) {
        update_prop_str(fm, FmProperty::Action, &mut self.0);
    }
}

impl FmComponent for FmStatusComponent {
    fn replicate_changes(&mut self, fm: &mut Mapping) {
        update_prop_str(fm, FmProperty::Status, &mut self.0);
    }
}

impl FmComponent for FmProjectComponent {
    fn replicate_changes(&mut self, fm: &mut Mapping) {
        update_prop_proj(fm, FmProperty::Project, &mut self.0);
    }
}
