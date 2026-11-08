use enum_iterator::Sequence;

use crate::vault::{ecs::EcsFileView, fm::{FmAction, FmProject, FmStatus, FmType}};


#[derive(Debug, Default)]
pub struct UiFilter {
    pub by_type:      FilterByList<FmType>,
    pub by_action:    FilterByList<FmAction>,
    pub by_status:    FilterByList<FmStatus>,
    pub by_project:   FilterByList<String>,
    pub needs_sorted: Option<bool>,
    pub is_open:      Option<bool>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum FilterByList<T> {
    All,
    None,
    Include(Vec<T>),
    Exclude(Vec<T>),
}

impl<Value> FilterByList<Value>
where
    Value: PartialEq
{
    pub fn matches(&self, value: Option<&Value>) -> bool {
        match self {
            FilterByList::All            => true,
            FilterByList::None           => value.is_none(),
            FilterByList::Include(items) => value.is_some_and(|v|  items.contains(v)),
            FilterByList::Exclude(items) => value.is_none_or (|v| !items.contains(v)),
        }
    }

    pub fn matches_by<Func, T>(&self, value: Option<&T>, selector: Func) -> bool
    where
        Func:  Fn(&T) -> &Value,
        Value: PartialEq,
    {
        match self {
            FilterByList::All            => true,
            FilterByList::None           => value.is_none(),
            FilterByList::Include(items) => value.is_some_and(|v|  items.iter().any(|i| i == selector(v))),
            FilterByList::Exclude(items) => value.is_none_or (|v| !items.iter().any(|i| i == selector(v))),
        }
    }
}

impl UiFilter {
    pub fn matches(&self, file: &EcsFileView) -> bool {
        all(&[
            self.by_type   .matches   (file.type_),
            self.by_action .matches   (file.action),
            self.by_status .matches   (file.status),
            self.by_project.matches_by(file.project, |p: &FmProject| &p.name),

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
        FilterByList::All
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "todo"]
    fn test_filters() {
        todo!()
    }

}
