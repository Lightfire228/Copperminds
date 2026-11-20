use yaml_serde::Mapping;

use crate::{config::Config, vault::{ecs::{File, FileId}, fm::*}};

pub trait Component: Send + 'static {
    #[allow(dead_code)]
    fn fm_component    (&    self) -> Option<&    dyn FmComponent> { None }
    fn fm_component_mut(&mut self) -> Option<&mut dyn FmComponent> { None }
}

pub trait FmComponent {
    fn replicate_changes(&self, fm: &mut Mapping);

    fn set_modified(&mut self, modified: bool);
}


#[derive(Debug)]
pub struct FmProp<T> {
    pub value:    Option<T>,
    pub modified: bool,
}

macro_rules! def_fm_component {
    ($name:ident, $kind:ident) => {

        #[derive(Debug)]
        pub struct $name(pub FmProp<$kind>);

    };
}

def_fm_component!(FmYamlComponent,    Mapping);
def_fm_component!(FmTypeComponent,    FmType);
def_fm_component!(FmActionComponent,  FmAction);
def_fm_component!(FmStatusComponent,  FmStatus);
def_fm_component!(FmProjectComponent, FmProject);
def_fm_component!(FmTagsComponent,    FmTags);

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

macro_rules! impl_component {
    ($ty:ty) => {
        impl Component for $ty {
            fn fm_component    (&    self) -> Option<&    dyn FmComponent> { None }

            fn fm_component_mut(&mut self) -> Option<&mut dyn FmComponent> { None }
        }
    };
}

impl_component!(FmYamlComponent);
impl_component!(MdTextComponent);
impl_component!(EmptyFileComponent);
impl_component!(IllegalNameComponent);


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


impl_component_with_fm!(FmTypeComponent);
impl_component_with_fm!(FmActionComponent);
impl_component_with_fm!(FmStatusComponent);
impl_component_with_fm!(FmProjectComponent);
impl_component_with_fm!(FmTagsComponent);

macro_rules! def_set_modified {
    () => {
        fn set_modified(&mut self, value: bool) {
            self.0.modified = value;
        }
    };
}


macro_rules! impl_fm_component_prop_str {
    ($name:ident, $kind:ident) => {

        impl FmComponent for $name {

            fn replicate_changes(&self, fm: &mut Mapping) {

                if !self.0.modified {
                    return;
                }

                update_prop_str(fm, FmProperty::$kind, self.0.value.as_ref());
            }

            def_set_modified!();
        }
    };
}

impl_fm_component_prop_str!(FmTypeComponent,   Type);
impl_fm_component_prop_str!(FmActionComponent, Action);
impl_fm_component_prop_str!(FmStatusComponent, Status);


impl FmComponent for FmProjectComponent {

    fn replicate_changes(&self, fm: &mut Mapping) {
        if !self.0.modified {
            return;
        }

        update_prop_proj(fm, FmProperty::Project, self.0.value.as_ref());
    }

    def_set_modified!();
}

impl FmComponent for FmTagsComponent {

    fn replicate_changes(&self, fm: &mut Mapping) {
        if !self.0.modified {
            return;
        }

        let tags = self.0
            .value
            .as_ref()
            .map   (|x| x.tags.as_slice())
        ;

        update_prop_list(fm, FmProperty::Tags, tags);
    }

    def_set_modified!();
}
