use crate::packs::Packs;
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
            active_menu_item: MenuItem::Summary,
            active_context_menu_item: ContextMenuItem::Info(0),
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
        } else {
            match self.menu_context.active_menu_item {
                MenuItem::Summary => self.menu_context.active_menu_item = MenuItem::Actions,
                MenuItem::Actions => self.menu_context.active_menu_item = MenuItem::Packs,
                MenuItem::Packs => self.menu_context.active_menu_item = MenuItem::Summary,
            }
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
        self.menu_context.active_context_menu_item = ContextMenuItem::NoViolationDependents(0);
    }

    pub fn handle_context_menu_v(&mut self) {
        self.menu_context.active_context_menu_item = ContextMenuItem::ViolationDependents(0);
    }

    pub fn handle_context_menu_i(&mut self) {
        self.menu_context.active_context_menu_item = ContextMenuItem::Info(0);
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
    Summary,
    Actions,
    Packs,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Summary => 0,
            MenuItem::Actions => 1,
            MenuItem::Packs => 2,
        }
    }
}

pub enum ActiveFocus<'a> {
    FilterPacks(TextArea<'a>),
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum ContextMenuItem {
    Info(usize),
    NoViolationDependents(usize),
    ViolationDependents(usize),
}

impl ContextMenuItem {
    pub fn set_scroll(&mut self, s: usize) {
        match self {
            ContextMenuItem::Info(scroll) => *scroll = s,
            ContextMenuItem::NoViolationDependents(scroll) => *scroll = s,
            ContextMenuItem::ViolationDependents(scroll) => *scroll = s,
        }
    }
    pub fn scroll(&self) -> usize {
        match self {
            ContextMenuItem::Info(scroll) => *scroll,
            ContextMenuItem::NoViolationDependents(scroll) => *scroll,
            ContextMenuItem::ViolationDependents(scroll) => *scroll,
        }
    }

    pub fn reset_scroll(&mut self) {
        self.set_scroll(0);
    }

    pub fn next_scroll(&mut self) {
        match self {
            ContextMenuItem::Info(scroll) => *scroll += 1,
            ContextMenuItem::NoViolationDependents(scroll) => *scroll += 1,
            ContextMenuItem::ViolationDependents(scroll) => *scroll += 1,
        }
    }

    pub fn previous_scroll(&mut self) {
        match self {
            ContextMenuItem::Info(scroll) => {
                if *scroll > 0 {
                    *scroll -= 1;
                }
            }
            ContextMenuItem::NoViolationDependents(scroll) => {
                if *scroll > 0 {
                    *scroll -= 1;
                }
            }
            ContextMenuItem::ViolationDependents(scroll) => {
                if *scroll > 0 {
                    *scroll -= 1;
                }
            }
        }
    }
}

impl From<ContextMenuItem> for usize {
    fn from(input: ContextMenuItem) -> usize {
        match input {
            ContextMenuItem::Info(_scroll) => 0,
            ContextMenuItem::NoViolationDependents(_scroll) => 1,
            ContextMenuItem::ViolationDependents(_scroll) => 2,
        }
    }
}
