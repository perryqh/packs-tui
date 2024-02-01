use crate::components::helpers::violations_display::VIOLATION_HEADER_ABBR_TITLES;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ScrollSortable {
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
    pub sort_column: usize,
    pub focused_column: usize, // so it can be active without sorting
    pub sort_direction: SortDirection,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Descending
    }
}

impl ScrollSortable {
    pub fn next_focus_column(&mut self) {
        self.focused_column += 1;
        if self.focused_column >= VIOLATION_HEADER_ABBR_TITLES.len() {
            self.focused_column = 0
        }
    }
    pub fn focused_column(&self) -> usize {
        self.focused_column
    }

    pub fn is_sort_ascending(&self) -> bool {
        self.sort_direction == SortDirection::Ascending
    }
    pub fn sort_column(&self) -> usize {
        self.sort_column
    }

    pub fn set_sort_column_to_active_column(&mut self) {
        self.sort_column = self.focused_column;
    }

    pub fn set_vertical_scroll(&mut self, scroll: usize) {
        self.vertical_scroll = scroll;
    }

    pub fn vertical_scroll(&self) -> usize {
        self.vertical_scroll
    }

    pub fn reset_vertical_scroll(&mut self) {
        self.set_vertical_scroll(0);
    }

    pub fn next_vertical_scroll(&mut self) {
        self.vertical_scroll += 1;
    }

    pub fn previous_vertical_scroll(&mut self) {
        if self.vertical_scroll > 0 {
            self.vertical_scroll -= 1;
        }
    }

    pub fn set_horizontal_scroll(&mut self, scroll: usize) {
        self.horizontal_scroll = scroll;
    }

    pub fn horizontal_scroll(&self) -> usize {
        self.horizontal_scroll
    }

    pub fn reset_horizontal_scroll(&mut self) {
        self.set_horizontal_scroll(0);
    }

    pub fn next_horizontal_scroll(&mut self) {
        self.horizontal_scroll += 1;
    }

    pub fn previous_horizontal_scroll(&mut self) {
        if self.horizontal_scroll > 0 {
            self.horizontal_scroll -= 1;
        }
    }

    pub fn sort_ascending(&mut self) {
        self.sort_direction = SortDirection::Ascending;
    }

    pub fn sort_descending(&mut self) {
        self.sort_direction = SortDirection::Descending;
    }
}
