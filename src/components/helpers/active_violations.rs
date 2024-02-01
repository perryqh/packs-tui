use crate::components::helpers::scroll_sortable::{ScrollSortable, SortDirection};
use crate::components::helpers::violations_display::VIOLATION_HEADER_ABBR_TITLES;
use serde::{Deserialize, Serialize};

pub const UNCONTAINED_OUT_SORTABLE: usize = 0;
pub const UNCONTAINED_IN_SORTABLE: usize = 1;
pub const CONTAINED_OUT_SORTABLE: usize = 2;
pub const CONTAINED_IN_SORTABLE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActiveViolations {
    Out,
    In,
    ContainedOut,
    ContainedIn,
}

impl Default for ActiveViolations {
    fn default() -> Self {
        Self::Out
    }
}
