use enum_iterator::Sequence;

use crate::vault::{ecs::EcsFileView, fm::{FmAction, FmStatus, FmType}};


#[derive(Debug, Default)]
pub struct UiFilter {
    pub by_type:      FilterByList<FmType>,
    pub by_action:    FilterByList<FmAction>,
    pub by_status:    FilterByList<FmStatus>,
    pub needs_sorted: Option<bool>,
    pub is_open:      Option<bool>,
}

#[derive(Debug)]
pub enum FilterByList<T> {
    None,
    Include(Vec<T>),

    #[allow(dead_code)]
    Exclude(Vec<T>),
}

impl<T> FilterByList<T>
where
    T: Sequence
{
    pub fn _all() -> Self {
        Self::Include(enum_iterator::all().collect())
    }

    pub fn _none() -> Self {
        Self::Exclude(enum_iterator::all().collect())
    }
}

impl<T> FilterByList<T>
where
    T: PartialEq
{
    pub fn matches(&self, value: Option<&T>) -> bool {
        match self {
            FilterByList::Include(items) => value.is_some_and(|v|  items.contains(v)),
            FilterByList::Exclude(items) => value.is_none_or (|v| !items.contains(v)),
            FilterByList::None           => true,
        }
    }
}

impl UiFilter {
    pub fn matches(&self, file: &EcsFileView) -> bool {
        all(&[
            self.by_type  .matches(file.type_),
            self.by_action.matches(file.action),
            self.by_status.matches(file.status),

            self.needs_sorted.is_none_or(|x| x == file.needs_sorting()),
            self.is_open     .is_none_or(|x| x == file.is_open      ()),

        ])
    }
}

fn all(values: &[bool]) -> bool {
    values.iter().all(|x| *x)
}


impl<T> Default for FilterByList<T> {
    fn default() -> Self {
        FilterByList::None
    }
}
