use crate::packs::{
    ConstantSummary, PackDependentViolation, Packs, DEPENDENT_PACK_VIOLATION_COUNT_HEADERS,
};
use anyhow::Result;
use crossterm::event::KeyEvent;
use std::rc::Rc;
use tui_textarea::TextArea;

/// Application result type.
pub type AppResult<T> = Result<T>;

/// Application.
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    pub packs: Packs,
    pub menu_context: MenuContext<'a>,
}

pub struct MenuContext<'a> {
    pub active_menu_item: MenuItem,
    pub active_context_menu_item: ContextMenuItem,
    pub active_focus: ActiveFocus<'a>,
}

impl Default for MenuContext<'_> {
    fn default() -> Self {
        Self {
            active_menu_item: MenuItem::Packs,
            active_context_menu_item: ContextMenuItem::Info(ContextMenuInfo::default()),
            active_focus: ActiveFocus::Left,
        }
    }
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            running: true,
            packs: Packs::default(),
            menu_context: MenuContext::default(),
        }
    }
}

impl App<'_> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn unselect(&mut self) {
        self.packs.unselect_pack_list();
    }

    pub fn sort_descending(&mut self) {
        self.menu_context.active_context_menu_item.sort_descending();
    }

    pub fn sort_ascending(&mut self) {
        self.menu_context.active_context_menu_item.sort_ascending();
    }

    pub fn next(&mut self) {
        match self.menu_context.active_focus {
            ActiveFocus::Left => match self.menu_context.active_menu_item {
                MenuItem::Packs => {
                    self.menu_context.active_context_menu_item.reset_scroll();
                    self.packs.next_pack_list();
                }
                MenuItem::Actions => {}
                MenuItem::Summary => {
                    if let ContextMenuItem::ConstantViolations(_) =
                        self.menu_context.active_context_menu_item
                    {
                        self.menu_context.active_context_menu_item.next_scroll();
                    }
                }
            },
            ActiveFocus::Right => {
                self.menu_context.active_context_menu_item.next_scroll();
            }
            ActiveFocus::Filter(_) => {
                self.menu_context.active_focus = ActiveFocus::Left;
            }
        }
    }

    pub fn previous(&mut self) {
        match self.menu_context.active_focus {
            ActiveFocus::Left => match self.menu_context.active_menu_item {
                MenuItem::Packs => {
                    self.menu_context.active_context_menu_item.reset_scroll();
                    self.packs.previous_pack_list();
                }
                MenuItem::Actions => {}
                MenuItem::Summary => {
                    if let ContextMenuItem::ConstantViolations(_) =
                        self.menu_context.active_context_menu_item
                    {
                        self.menu_context.active_context_menu_item.previous_scroll();
                    }
                }
            },
            ActiveFocus::Right => {
                self.menu_context.active_context_menu_item.previous_scroll();
            }
            ActiveFocus::Filter(_) => {}
        }
    }

    pub fn handle_tab(&mut self) {
        if let ActiveFocus::Filter(_) = self.menu_context.active_focus {
            self.menu_context.active_focus = ActiveFocus::Left;
        } else if let ContextMenuItem::ViolationDependents(ref mut violation) =
            self.menu_context.active_context_menu_item
        {
            violation.next_sort_column();
        } else if let ContextMenuItem::ConstantViolations(ref mut violation) =
            self.menu_context.active_context_menu_item
        {
            violation.next_sort_column();
        }
    }

    pub fn handle_top_menu_s(&mut self) {
        self.menu_context.active_menu_item = MenuItem::Summary;
        self.menu_context.active_context_menu_item =
            ContextMenuItem::Info(ContextMenuInfo::default());
    }

    pub fn handle_top_menu_p(&mut self) {
        self.menu_context.active_menu_item = MenuItem::Packs;
        self.menu_context.active_context_menu_item =
            ContextMenuItem::Info(ContextMenuInfo::default());
    }

    pub fn handle_top_menu_a(&mut self) {
        self.menu_context.active_menu_item = MenuItem::Actions;
    }

    pub fn handle_context_menu_d(&mut self) {
        self.menu_context.active_context_menu_item =
            ContextMenuItem::NoViolationDependents(ContextMenuNoViolationDependents::default());
    }

    pub fn handle_context_menu_c(&mut self) {
        self.menu_context.active_context_menu_item =
            ContextMenuItem::ConstantViolations(ContextMenuConstantViolations::default());
    }

    pub fn handle_context_menu_v(&mut self) {
        self.menu_context.active_context_menu_item =
            ContextMenuItem::ViolationDependents(ContextMenuViolationDependents::default());
    }

    pub fn handle_context_menu_i(&mut self) {
        self.menu_context.active_context_menu_item =
            ContextMenuItem::Info(ContextMenuInfo::default());
    }

    pub fn focus_left(&mut self) {
        self.menu_context.active_focus = ActiveFocus::Left;
    }

    pub fn focus_right(&mut self) {
        self.menu_context.active_focus = ActiveFocus::Right;
    }

    pub fn focus_filter(&mut self) {
        let text = match &self.menu_context.active_menu_item {
            MenuItem::Summary => {
                if let Some(summaries) = &self.packs.constant_violation_summaries {
                    summaries.filter.clone()
                } else {
                    "".to_string()
                }
            }
            MenuItem::Packs => {
                if let Some(pack_list) = &self.packs.pack_list {
                    pack_list.filter.clone()
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        };
        self.menu_context.active_focus = ActiveFocus::Filter(TextArea::new(vec![text]));
    }

    pub fn handle_as_textarea(&mut self, key_event: KeyEvent) -> bool {
        if let ActiveFocus::Filter(ref mut textarea) = self.menu_context.active_focus {
            textarea.input(key_event);
            let filter = textarea.lines().join("");
            self.update_filter(filter);

            return true;
        }
        false
    }

    pub fn update_filter(&mut self, filter: String) {
        match (
            self.menu_context.active_menu_item,
            self.menu_context.active_context_menu_item,
        ) {
            (MenuItem::Packs, _) => {
                if let Some(ref mut pack_list) = self.packs.pack_list {
                    pack_list.filter = filter;
                }
            }
            (MenuItem::Summary, ContextMenuItem::ConstantViolations(_)) => {
                if let Some(ref mut constant) = self.packs.constant_violation_summaries {
                    constant.filter = filter;
                }
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Packs,
    Actions,
    Summary,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Packs => 0,
            MenuItem::Actions => 1,
            MenuItem::Summary => 2,
        }
    }
}

pub enum ActiveFocus<'a> {
    Filter(TextArea<'a>),
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ContextMenuInfo {
    pub scroll: usize,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ContextMenuNoViolationDependents {
    pub scroll: usize,
}

pub const DEPENDENT_PACK_VIOLATION_HEADER_ABBR_TITLES: [&str; 7] =
    ["pack", "cnst", "arch", "dep", "fvis", "priv", "vis"];
pub const DEPENDENT_PACK_VIOLATION_HEADER_FULL_TITLES: [&str; 7] = [
    "pack",
    "constant",
    "architecture",
    "dependency",
    "folder visibility",
    "privacy",
    "visibility",
];

pub const CONSTANT_VIOLATION_COLUMNS: [&str; 8] = [
    "constant",
    "count",
    "architecture",
    "dependency",
    "folder visibility",
    "privacy",
    "visibility",
    "ref packs",
];

impl ContextMenuConstantViolations {
    pub fn next_sort_column(&mut self) {
        self.sort_column += 1;
        if self.sort_column > CONSTANT_VIOLATION_COLUMNS.len() - 1 {
            self.sort_column = 0;
        }
    }

    pub fn header_titles(&mut self) -> Vec<String> {
        CONSTANT_VIOLATION_COLUMNS
            .iter()
            .enumerate()
            .map(|(i, title)| {
                if i == self.sort_column {
                    format!(
                        "{} {}",
                        if self.sort_direction == SortDirection::Descending {
                            "▲"
                        } else {
                            "▼"
                        },
                        title
                    )
                } else {
                    title.to_string()
                }
            })
            .collect::<Vec<String>>()
    }

    pub(crate) fn sort_violations(&mut self, violations: &mut [Rc<ConstantSummary>]) {
        match self.sort_column {
            0 => violations.sort_by(|a, b| a.constant.cmp(&b.constant)),
            1 => violations.sort_by(|a, b| {
                let result = a.count.cmp(&b.count);
                if result == std::cmp::Ordering::Equal {
                    a.constant.cmp(&b.constant)
                } else {
                    result
                }
            }),
            2..=6 => violations.sort_by(|a, b| {
                let result = a
                    .count_for_violation_type(CONSTANT_VIOLATION_COLUMNS[self.sort_column])
                    .cmp(&b.count_for_violation_type(CONSTANT_VIOLATION_COLUMNS[self.sort_column]));
                if result == std::cmp::Ordering::Equal {
                    a.constant.cmp(&b.constant)
                } else {
                    result
                }
            }),
            7 => violations.sort_by(|a, b| {
                let result = a
                    .referencing_pack_count_length()
                    .cmp(&b.referencing_pack_count_length());
                if result == std::cmp::Ordering::Equal {
                    a.constant.cmp(&b.constant)
                } else {
                    result
                }
            }),
            _ => {}
        }
        if let SortDirection::Descending = self.sort_direction {
            violations.reverse();
        }
    }
}

impl ContextMenuViolationDependents {
    pub fn next_sort_column(&mut self) {
        self.sort_column += 1;
        if self.sort_column > DEPENDENT_PACK_VIOLATION_HEADER_ABBR_TITLES.len() - 1 {
            self.sort_column = 0;
        }
    }

    pub fn header_titles(&mut self) -> Vec<String> {
        DEPENDENT_PACK_VIOLATION_HEADER_ABBR_TITLES
            .iter()
            .enumerate()
            .map(|(i, title)| {
                if i == self.sort_column {
                    format!(
                        "{} {}",
                        if self.sort_direction == SortDirection::Descending {
                            "▲"
                        } else {
                            "▼"
                        },
                        DEPENDENT_PACK_VIOLATION_HEADER_FULL_TITLES[i]
                    )
                } else {
                    title.to_string()
                }
            })
            .collect::<Vec<String>>()
    }

    pub fn sort_violations(&mut self, violations: &mut [&PackDependentViolation]) {
        match self.sort_column {
            0 => violations.sort_by(|a, b| a.defining_pack_name.cmp(&b.defining_pack_name)),
            2..=6 => violations.sort_by(|a, b| {
                a.count_for_violation_type(
                    DEPENDENT_PACK_VIOLATION_COUNT_HEADERS[self.sort_column - 2],
                )
                .cmp(&b.count_for_violation_type(
                    DEPENDENT_PACK_VIOLATION_COUNT_HEADERS[self.sort_column - 2],
                ))
            }),
            1 => violations.sort_by_key(|a| a.num_constants()),
            _ => {}
        }
        if let SortDirection::Descending = self.sort_direction {
            violations.reverse();
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ContextMenuViolationDependents {
    pub scroll: usize,
    pub sort_column: usize,
    pub sort_direction: SortDirection,
}

#[derive(Copy, Clone, Debug)]
pub struct ContextMenuConstantViolations {
    pub scroll: usize,
    pub sort_column: usize,
    pub sort_direction: SortDirection,
}

#[derive(Copy, Clone, Debug)]
pub enum ContextMenuItem {
    Info(ContextMenuInfo),
    NoViolationDependents(ContextMenuNoViolationDependents),
    ViolationDependents(ContextMenuViolationDependents),
    ConstantViolations(ContextMenuConstantViolations),
}

impl Default for ContextMenuViolationDependents {
    fn default() -> Self {
        Self {
            scroll: 0,
            sort_column: 0,
            sort_direction: SortDirection::Ascending,
        }
    }
}

impl Default for ContextMenuConstantViolations {
    fn default() -> Self {
        Self {
            scroll: 0,
            sort_column: 0,
            sort_direction: SortDirection::Ascending,
        }
    }
}

impl ContextMenuItem {
    pub fn set_scroll(&mut self, s: usize) {
        match self {
            ContextMenuItem::Info(info) => info.scroll = s,
            ContextMenuItem::NoViolationDependents(no_violation) => no_violation.scroll = s,
            ContextMenuItem::ViolationDependents(violation) => violation.scroll = s,
            ContextMenuItem::ConstantViolations(constant_violations) => {
                constant_violations.scroll = s
            }
        }
    }
    pub fn scroll(&self) -> usize {
        match self {
            ContextMenuItem::Info(info) => info.scroll,
            ContextMenuItem::NoViolationDependents(no_violation) => no_violation.scroll,
            ContextMenuItem::ViolationDependents(violation) => violation.scroll,
            ContextMenuItem::ConstantViolations(violation) => violation.scroll,
        }
    }

    pub fn reset_scroll(&mut self) {
        self.set_scroll(0);
    }

    pub fn next_scroll(&mut self) {
        match self {
            ContextMenuItem::Info(info) => info.scroll += 1,
            ContextMenuItem::NoViolationDependents(no_violation) => no_violation.scroll += 1,
            ContextMenuItem::ViolationDependents(violation) => violation.scroll += 1,
            ContextMenuItem::ConstantViolations(violation) => violation.scroll += 1,
        }
    }

    pub fn previous_scroll(&mut self) {
        match self {
            ContextMenuItem::Info(info) => {
                if info.scroll > 0 {
                    info.scroll -= 1;
                }
            }
            ContextMenuItem::NoViolationDependents(no_violation) => {
                if no_violation.scroll > 0 {
                    no_violation.scroll -= 1;
                }
            }
            ContextMenuItem::ViolationDependents(violation) => {
                if violation.scroll > 0 {
                    violation.scroll -= 1;
                }
            }
            ContextMenuItem::ConstantViolations(violation) => {
                if violation.scroll > 0 {
                    violation.scroll -= 1;
                }
            }
        }
    }

    pub fn sort_ascending(&mut self) {
        match self {
            ContextMenuItem::Info(_) => {}
            ContextMenuItem::NoViolationDependents(_) => {}
            ContextMenuItem::ViolationDependents(violation) => {
                violation.sort_direction = SortDirection::Ascending;
            }
            ContextMenuItem::ConstantViolations(violation) => {
                violation.sort_direction = SortDirection::Ascending;
            }
        }
    }

    pub fn sort_descending(&mut self) {
        match self {
            ContextMenuItem::Info(_) => {}
            ContextMenuItem::NoViolationDependents(_) => {}
            ContextMenuItem::ViolationDependents(violation) => {
                violation.sort_direction = SortDirection::Descending;
            }
            ContextMenuItem::ConstantViolations(violation) => {
                violation.sort_direction = SortDirection::Descending;
            }
        }
    }
}

impl From<ContextMenuItem> for usize {
    fn from(input: ContextMenuItem) -> usize {
        match input {
            ContextMenuItem::Info(_) => 0,
            ContextMenuItem::NoViolationDependents(_) => 1,
            ContextMenuItem::ViolationDependents(_) => 2,
            ContextMenuItem::ConstantViolations(_) => 3,
        }
    }
}
