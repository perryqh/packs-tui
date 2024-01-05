use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        // Top Menu handlers
        KeyCode::Tab => app.handle_tab(),
        KeyCode::Char('s') => app.handle_top_menu_s(),
        KeyCode::Char('p') => app.handle_top_menu_p(),
        KeyCode::Char('a') => app.handle_top_menu_a(),

        // Counter handlers
        KeyCode::Left | KeyCode::Char('h') => app.focus_left(),
        KeyCode::Right | KeyCode::Char('l') => app.focus_right(),
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        // Content menu handlers
        KeyCode::Char('d') => app.handle_context_menu_d(),
        KeyCode::Char('i') => app.handle_context_menu_i(),
        _ => {}
    }
    Ok(())
}
