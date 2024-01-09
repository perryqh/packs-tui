use crate::app::{App, MenuItem};
use ratatui::Frame;

mod actions;
mod packs;
mod shared;
mod summary;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.menu_context.active_menu_item {
        MenuItem::Summary => summary::render_summary(app, frame),
        MenuItem::Actions => actions::render_actions(app, frame),
        MenuItem::Packs => packs::render_packs(app, frame),
    }
}
