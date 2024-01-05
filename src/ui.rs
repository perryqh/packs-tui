use std::rc::Rc;
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::block::{Title};

use crate::app::{ActiveFocus, App, ContextMenuItem, MenuItem};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.menu_context.active_menu_item {
        MenuItem::Summary => render_summary(app, frame),
        MenuItem::Actions => render_actions(app, frame),
        MenuItem::Packs => render_packs(app, frame),
    }
}

fn render_summary(app: &mut App, frame: &mut Frame) {
    let chunks = build_chunks(frame);
    let top_menu_tabs = build_top_menu(&app);
    frame.render_widget(top_menu_tabs, chunks[0]);
}
fn render_actions(app: &mut App, frame: &mut Frame) {
    let chunks = build_chunks(frame);
    let top_menu_tabs = build_top_menu(&app);
    frame.render_widget(top_menu_tabs, chunks[0]);
}
fn render_packs(app: &mut App, frame: &mut Frame) {
    let chunks = build_chunks(frame);
    let top_menu_tabs = build_top_menu(&app);
    frame.render_widget(top_menu_tabs, chunks[0]);

    let (title, pack_name, vertical_scroll, pack_info) = match app.packs.get_pack_list().selected_pack() {
        Some(pack) => {
            match app.menu_context.active_context_menu_item {
                ContextMenuItem::Info(scroll) => {
                    (String::from("info"), pack.name.clone(), scroll, app.packs.pack_info(&pack))
                },
                ContextMenuItem::Dependents(scroll) => {
                    let dependents = app.packs.pack_dependents(&pack);
                    (String::from("dependents"), pack.name.clone(), scroll, format!("{:?}", dependents))
                }
            }
        },
        None => (String::from("nope"), "".to_string(), 0,"".to_string()),
    };

    let title_block = Block::new()
        .title(Span::styled(title,
                            Style::default()
                                .add_modifier(Modifier::BOLD)))
        .title(Title::from(pack_name).alignment(Alignment::Right))
        .borders(Borders::ALL);

    match app.menu_context.active_focus {
        ActiveFocus::Left => {
            let outer_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(33), Constraint::Percentage(67)])
                .split(chunks[1]);

            frame.render_widget(
                Paragraph::new(pack_info.as_str())
                    .scroll((vertical_scroll as u16, 0))
                    .block(title_block),
                outer_layout[1],
            );

            let pack_list = app.packs.get_pack_list();

            let list_items: Vec<ListItem> = pack_list
                .items
                .iter()
                .map(|pack| ListItem::new(pack.name.clone()))
                .collect();

            let items = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("packs"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_stateful_widget(items, outer_layout[0], &mut pack_list.state);
        },
        ActiveFocus::Right => {
            frame.render_widget(
                Paragraph::new(pack_info.as_str())
                    .scroll((vertical_scroll as u16, 0))
                    .block(title_block),
                chunks[1],
            );
        },
    }

    let menu_titles = vec!["Info".to_string(), "Dependents".to_string()];
    match build_context_menu(&app, &menu_titles) {
        Some(context_menu_tabs) => frame.render_widget(context_menu_tabs, chunks[2]),
        None => {}
    }
}

fn build_chunks(frame: &mut Frame) -> Rc<[Rect]> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
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

fn build_top_menu(app: &App) -> Tabs {
    let menu_titles = vec!["Summary", "Actions", "Packs"];
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

fn build_context_menu<'a>(app: &App, menu_titles: &'a Vec<String>) -> Option<Tabs<'a>> {
    let menu: Vec<Line> = menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect();
    let tabs = Tabs::new(menu)
        .select(app.menu_context.active_context_menu_item.into())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::LightBlue))
        .divider(Span::raw("|"));
    Some(tabs)
}
