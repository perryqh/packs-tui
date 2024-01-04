use crate::packs::Packs;
use anyhow::Result;

/// Application result type.
pub type AppResult<T> = Result<T>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub packs: Packs,
    pub menu_context: MenuContext,
}

pub struct MenuContext {
    pub active_menu_item: MenuItem,
    pub active_context_menu_item: ContextMenuItem,
}

impl Default for MenuContext {
    fn default() -> Self {
        Self {
            active_menu_item: MenuItem::Summary,
            active_context_menu_item: ContextMenuItem::Info,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            packs: Packs::default(),
            menu_context: MenuContext::default(),
        }
    }
}

impl App {
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
        self.packs.next_pack_list();
    }

    pub fn previous(&mut self) {
        self.packs.previous_pack_list();
    }

    pub fn handle_tab(&mut self) {
        match self.menu_context.active_menu_item {
            MenuItem::Summary => self.menu_context.active_menu_item = MenuItem::Actions,
            MenuItem::Actions => self.menu_context.active_menu_item = MenuItem::Packs,
            MenuItem::Packs => self.menu_context.active_menu_item = MenuItem::Summary,
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
        self.menu_context.active_context_menu_item = ContextMenuItem::Dependents;
    }

    pub fn handle_context_menu_i(&mut self) {
        self.menu_context.active_context_menu_item = ContextMenuItem::Info;
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

#[derive(Copy, Clone, Debug)]
pub enum ContextMenuItem {
    Info,
    Dependents,
}

impl From<ContextMenuItem> for usize {
    fn from(input: ContextMenuItem) -> usize {
        match input {
            ContextMenuItem::Info => 0,
            ContextMenuItem::Dependents => 1,
        }
    }
}
