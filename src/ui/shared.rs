use crate::app::{ActiveFocus, App};
use ratatui::{prelude::*, widgets::*};
use std::rc::Rc;
use tui_textarea::CursorMove;

pub fn build_top_menu<'a>(app: &'a App<'a>) -> Tabs<'a> {
    let menu_titles = ["Packs", "Actions", "Summary"];
    let menu = menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect();
    let tabs = Tabs::new(menu)
        .select(app.menu_context.active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));
    tabs
}

pub fn build_context_menu(menu_titles: &[String], selected: usize) -> Option<Tabs> {
    let menu: Vec<Line> = menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect();
    let tabs = Tabs::new(menu)
        .select(selected)
        .block(Block::default().title("Context Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Gray).fg(Color::Black))
        .divider(Span::raw("|"));
    Some(tabs)
}

pub fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn build_chunks(frame: &mut Frame) -> Rc<[Rect]> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.size());
    chunks
}

pub fn render_filter_textarea(
    active_focus: &mut ActiveFocus,
    frame: &mut Frame,
    rect: Rect,
    filter_text: &str,
) {
    let filter_title_block = Block::default().borders(Borders::BOTTOM);
    frame.render_widget(filter_title_block, rect);

    match active_focus {
        ActiveFocus::Filter(ref mut textarea) => {
            textarea.set_cursor_line_style(Style::default().fg(Color::Cyan));
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .fg(Color::Cyan)
                    .title("filter")
                    .title_alignment(Alignment::Right),
            );
            textarea.start_selection();
            textarea.move_cursor(CursorMove::WordForward);
            let widget = textarea.widget();
            frame.render_widget(widget, rect);
        }
        _ => {
            let mut existing_filter = filter_text;
            if existing_filter.is_empty() {
                existing_filter = "ctrl-f";
            }
            let line = Line::from(vec![Span::styled(
                existing_filter,
                Style::default().fg(Color::White).bold(),
            )]);

            let paragraph = Paragraph::new(line)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("filter")
                        .title_alignment(Alignment::Right),
                )
                .alignment(Alignment::Left);
            frame.render_widget(paragraph, rect);
        }
    }
}
