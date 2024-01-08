use crate::packs::{PackDependentViolation, Packs, DEPENDENT_PACK_VIOLATION_COUNT_HEADERS};
use anyhow::Result;
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

    pub fn next(&mut self) {
        if let ActiveFocus::FilterPacks(_) = self.menu_context.active_focus {
            self.menu_context.active_focus = ActiveFocus::Left;
        } else {
            match self.menu_context.active_focus {
                ActiveFocus::Left => {
                    self.menu_context.active_context_menu_item.reset_scroll();
                    self.packs.next_pack_list();
                }
                ActiveFocus::Right => {
                    self.menu_context.active_context_menu_item.next_scroll();
                }
                ActiveFocus::FilterPacks(_) => {
                    self.menu_context.active_focus = ActiveFocus::Left;
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.menu_context.active_focus {
            ActiveFocus::Left => {
                self.menu_context.active_context_menu_item.reset_scroll();
                self.packs.previous_pack_list();
            }
            ActiveFocus::Right => {
                self.menu_context.active_context_menu_item.previous_scroll();
            }
            ActiveFocus::FilterPacks(_) => {}
        }
    }

    pub fn handle_tab(&mut self) {
        if let ActiveFocus::FilterPacks(_) = self.menu_context.active_focus {
            self.menu_context.active_focus = ActiveFocus::Left;
        } else if let ContextMenuItem::ViolationDependents(ref mut violation) =
            self.menu_context.active_context_menu_item
        {
            violation.next_sort_column();
        }
    }

    pub fn handle_top_menu_s(&mut self) {
        self.menu_context.active_menu_item = MenuItem::Summary;
    }

    pub fn handle_top_menu_p(&mut self) {
        self.menu_context.active_menu_item = MenuItem::Packs;
    }

    pub fn handle_top_menu_a(&mut self) {
        self.menu_context.active_menu_item = MenuItem::Actions;
    }

    pub fn handle_context_menu_d(&mut self) {
        self.menu_context.active_context_menu_item =
            ContextMenuItem::NoViolationDependents(ContextMenuNoViolationDependents::default());
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

    pub fn focus_filter_packs(&mut self) {
        let text = match self.packs.pack_list {
            Some(ref pack_list) => pack_list.filter.clone(),
            None => "".to_string(),
        };
        self.menu_context.active_focus = ActiveFocus::FilterPacks(TextArea::new(vec![text]));
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
    FilterPacks(TextArea<'a>),
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

impl ContextMenuViolationDependents {
    pub fn next_sort_column(&mut self) {
        self.sort_column += 1;
        if self.sort_column > DEPENDENT_PACK_VIOLATION_HEADER_ABBR_TITLES.len() - 1 {
            self.sort_column = 0;
        }

        // TODO: remove hardcoded sort direction logic
        if self.sort_column == 0 {
            self.sort_direction = SortDirection::Ascending;
        } else {
            self.sort_direction = SortDirection::Descending;
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
            1..=5 => violations.sort_by(|a, b| {
                a.count_for_violation_type(
                    DEPENDENT_PACK_VIOLATION_COUNT_HEADERS[self.sort_column - 1],
                )
                .cmp(&b.count_for_violation_type(
                    DEPENDENT_PACK_VIOLATION_COUNT_HEADERS[self.sort_column - 1],
                ))
            }),
            6 => violations.sort_by_key(|a| a.num_constants()),
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
pub enum ContextMenuItem {
    Info(ContextMenuInfo),
    NoViolationDependents(ContextMenuNoViolationDependents),
    ViolationDependents(ContextMenuViolationDependents),
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

impl ContextMenuItem {
    pub fn set_scroll(&mut self, s: usize) {
        match self {
            ContextMenuItem::Info(info) => info.scroll = s,
            ContextMenuItem::NoViolationDependents(no_violation) => no_violation.scroll = s,
            ContextMenuItem::ViolationDependents(violation) => violation.scroll = s,
        }
    }
    pub fn scroll(&self) -> usize {
        match self {
            ContextMenuItem::Info(info) => info.scroll,
            ContextMenuItem::NoViolationDependents(no_violation) => no_violation.scroll,
            ContextMenuItem::ViolationDependents(violation) => violation.scroll,
        }
    }

    pub fn reset_scroll(&mut self) {
        self.set_scroll(0);
    }

    pub fn next_scroll(&mut self) {
        match self {
            ContextMenuItem::Info(info) => info.scroll += 1,
            ContextMenuItem::NoViolationDependents(no_violation) => no_violation.scroll += 1,
            ContextMenuItem::ViolationDependents(violation) => {
                violation.scroll += 1;
            }
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
        }
    }
}

impl From<ContextMenuItem> for usize {
    fn from(input: ContextMenuItem) -> usize {
        match input {
            ContextMenuItem::Info(_) => 0,
            ContextMenuItem::NoViolationDependents(_) => 1,
            ContextMenuItem::ViolationDependents(_) => 2,
        }
    }
}
