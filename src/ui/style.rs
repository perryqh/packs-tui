use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::PathBuf, rc::Rc};

pub type SharedTheme = Rc<Theme>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Theme {
    selected_tab: Color,
    command_fg: Color,
    selection_bg: Color,
    selection_fg: Color,
    cmdbar_bg: Color,
    cmdbar_extra_lines_bg: Color,
    disabled_fg: Color,
    danger_fg: Color,
    uncontained_in_violations_count_fg: Color,
    uncontained_out_violations_count_fg: Color,
}

impl Theme {
    pub fn scroll_bar_pos(&self) -> Style {
        Style::default().fg(self.selection_bg)
    }

    pub fn block(&self, focus: bool) -> Style {
        if focus {
            Style::default()
        } else {
            Style::default().fg(self.disabled_fg)
        }
    }

    pub fn title(&self, focused: bool) -> Style {
        if focused {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.disabled_fg)
        }
    }

    pub fn tab(&self, selected: bool) -> Style {
        if selected {
            self.text(true, false)
                .fg(self.selected_tab)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            self.text(false, false)
        }
    }

    pub fn text(&self, enabled: bool, selected: bool) -> Style {
        match (enabled, selected) {
            (false, false) => Style::default().fg(self.disabled_fg),
            (false, true) => Style::default().bg(self.selection_bg),
            (true, false) => Style::default(),
            (true, true) => Style::default().fg(self.command_fg).bg(self.selection_bg),
        }
    }

    fn apply_select(&self, style: Style, selected: bool) -> Style {
        if selected {
            style.bg(self.selection_bg).fg(self.selection_fg)
        } else {
            style
        }
    }

    pub fn text_danger(&self) -> Style {
        Style::default().fg(self.danger_fg)
    }

    pub fn commandbar(&self, enabled: bool, line: usize) -> Style {
        if enabled {
            Style::default().fg(self.command_fg)
        } else {
            Style::default().fg(self.disabled_fg)
        }
        .bg(if line == 0 {
            self.cmdbar_bg
        } else {
            self.cmdbar_extra_lines_bg
        })
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            selected_tab: Color::Reset,
            command_fg: Color::White,
            selection_bg: Color::Blue,
            selection_fg: Color::White,
            cmdbar_bg: Color::Blue,
            cmdbar_extra_lines_bg: Color::Blue,
            disabled_fg: Color::DarkGray,
            danger_fg: Color::Red,
            uncontained_in_violations_count_fg: Color::Yellow,
            uncontained_out_violations_count_fg: Color::Red,
        }
    }
}
