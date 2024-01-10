use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Tab => {
            app.handle_tab();
            return Ok(());
        }
        KeyCode::Esc => {
            app.focus_left();
            return Ok(());
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.sort_descending();
            } else {
                app.next();
            }
            return Ok(());
        }
        _ => {}
    }
    if app.handle_as_textarea(key_event) {
        return Ok(());
    }
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Char('q') => {
            app.quit();
        }

        // Top Menu handlers
        KeyCode::Char('s') => app.handle_top_menu_s(),
        KeyCode::Char('p') => app.handle_top_menu_p(),
        KeyCode::Char('a') => app.handle_top_menu_a(),

        KeyCode::Left => app.focus_left(),
        KeyCode::Right => app.focus_right(),
        KeyCode::Char('f') => app.focus_filter(),

        KeyCode::Up | KeyCode::Char('k') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.sort_ascending();
            } else {
                app.previous();
            }
        }

        // Content menu handlers
        KeyCode::Char('c') => app.handle_context_menu_c(),
        KeyCode::Char('d') => app.handle_context_menu_d(),
        KeyCode::Char('v') => app.handle_context_menu_v(),
        KeyCode::Char('i') => app.handle_context_menu_i(),
        _ => {}
    }
    Ok(())
}
