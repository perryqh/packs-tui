use crate::app::App;
use crate::ui::shared::{build_chunks, build_top_menu};
use ratatui::Frame;

pub fn render_actions(app: &mut App, frame: &mut Frame) {
    let chunks = build_chunks(frame);
    let top_menu_tabs = build_top_menu(app);
    frame.render_widget(top_menu_tabs, chunks[0]);
}
