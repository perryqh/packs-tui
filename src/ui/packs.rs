use ratatui::widgets::block::Title;
use ratatui::{prelude::*, widgets::*};
use tui_textarea::CursorMove;

use crate::app::{ActiveFocus, App, ContextMenuItem};
use crate::packs::DEPENDENT_PACK_VIOLATION_COUNT_HEADERS;
use crate::ui::shared::{build_chunks, build_context_menu, build_top_menu};

pub fn render_packs(app: &mut App, frame: &mut Frame) {
    let chunks = build_chunks(frame);
    let top_menu_tabs = build_top_menu(app);
    frame.render_widget(top_menu_tabs, chunks[0]);

    let funny = match app.packs.get_pack_list().selected_pack() {
        Some(_pack) => match app.menu_context.active_context_menu_item {
            ContextMenuItem::NoViolationDependents(_) => render_no_violation_dependents,
            ContextMenuItem::ViolationDependents(_) => render_violation_dependents,
            _ => render_info_context,
        },
        None => |_app: &mut App, _frame: &mut Frame, _rect: Rect| {},
    };

    match app.menu_context.active_focus {
        ActiveFocus::Left | ActiveFocus::Filter(_) => {
            let outer_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(33), Constraint::Percentage(67)])
                .split(chunks[1]);

            funny(app, frame, outer_layout[1]);

            render_pack_list(app, frame, outer_layout[0]);
        }
        ActiveFocus::Right => {
            funny(app, frame, chunks[1]);
        }
    }

    let menu_titles = vec![
        "Info".to_string(),
        "Dependents".to_string(),
        "Violation Dependents".to_string(),
    ];

    if let Some(context_menu_tabs) = build_context_menu(
        &menu_titles,
        app.menu_context.active_context_menu_item.into(),
    ) {
        frame.render_widget(context_menu_tabs, chunks[2])
    }
}

fn render_violation_dependents(app: &mut App, frame: &mut Frame, rect: Rect) {
    let pack_name = app
        .packs
        .get_pack_list()
        .selected_pack()
        .unwrap_or_else(|| panic!("no pack selected"))
        .name
        .clone();
    let content_menu_violation_dependents = match app.menu_context.active_context_menu_item {
        ContextMenuItem::ViolationDependents(ref mut content_menu_violation_dependents) => {
            content_menu_violation_dependents
        }
        _ => panic!("expected ContextMenuItem::ViolationDependents"),
    };
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_titles = &content_menu_violation_dependents.header_titles();
    let header_cells = header_titles
        .iter()
        .enumerate()
        .map(|(index, header_title)| {
            if content_menu_violation_dependents.sort_column == index {
                Cell::from(header_title.clone()).style(
                    Style::default()
                        .fg(Color::LightCyan)
                        .bg(Color::DarkGray)
                        .bold(),
                )
            } else {
                Cell::from(header_title.clone()).style(Style::default().fg(Color::LightCyan))
            }
        });
    let header = Row::new(header_cells).bold().height(1);
    let mut violations = app
        .packs
        .get_pack_dependent_violations_by_selected_defining_pack_name();
    content_menu_violation_dependents.sort_violations(&mut violations);

    let mut scroll = app.menu_context.active_context_menu_item.scroll();

    if scroll >= violations.len() && !violations.is_empty() {
        scroll = violations.len() - 1;
        app.menu_context.active_context_menu_item.set_scroll(scroll);
    }

    let mut table_state = TableState::default().with_selected(Some(scroll));
    let rows = violations.iter().map(|violation| {
        let height = 1;
        let mut cells = vec![];
        cells.push(Cell::from(violation.referencing_pack_name.clone()));
        cells.push(Cell::from(violation.num_constants().to_string()));
        for key in DEPENDENT_PACK_VIOLATION_COUNT_HEADERS {
            let count = violation.count_for_violation_type(key);
            cells.push(Cell::from(count.to_string()));
        }
        Row::new(cells).height(height as u16)
    });
    let max_len: usize = violations
        .iter()
        .map(|violation| violation.referencing_pack_name.len())
        .max()
        .unwrap_or(0);
    let mut widths = vec![Constraint::Length(max_len as u16)];
    header_titles
        .iter()
        .skip(1)
        .for_each(|h| widths.push(Constraint::Length(h.len() as u16)));
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Violation Dependents ({})", violations.len()))
                .title(Title::from(pack_name.clone()).alignment(Alignment::Right)),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ");
    frame.render_stateful_widget(table, rect, &mut table_state);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(violations.len())
        .position(scroll);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_stateful_widget(
        scrollbar,
        rect.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state,
    );
}

fn render_info_context(app: &mut App, frame: &mut Frame, rect: Rect) {
    let pack = app
        .packs
        .get_pack_list()
        .selected_pack()
        .unwrap_or_else(|| panic!("no pack selected"));
    let info = app.packs.pack_info(&pack);
    let lines = info.into_iter().map(Line::from).collect::<Vec<Line>>();
    let title_block = Block::new()
        .title(Span::styled(
            "info",
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .title(Title::from(pack.name.clone()).alignment(Alignment::Right))
        .borders(Borders::ALL);

    let content_length = lines.len();
    let mut scrollbar_state = ScrollbarState::new(content_length)
        .position(app.menu_context.active_context_menu_item.scroll());
    let paragraph = Paragraph::new(lines)
        .scroll((app.menu_context.active_context_menu_item.scroll() as u16, 0))
        .block(title_block);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    frame.render_widget(paragraph, rect);
    frame.render_stateful_widget(
        scrollbar,
        rect.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state,
    );
}

fn render_no_violation_dependents(app: &mut App, frame: &mut Frame, rect: Rect) {
    let pack = app
        .packs
        .get_pack_list()
        .selected_pack()
        .unwrap_or_else(|| panic!("no pack selected"));
    let dependents = app.packs.pack_dependents(&pack);
    let lines = dependents
        .into_iter()
        .map(Line::from)
        .collect::<Vec<Line>>();
    let title_block = Block::new()
        .title(Span::styled(
            format!("dependents ({})", lines.len()),
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .title(Title::from(pack.name.clone()).alignment(Alignment::Right))
        .borders(Borders::ALL);

    let content_length = lines.len();
    let scroll = app.menu_context.active_context_menu_item.scroll();
    let mut scrollbar_state = ScrollbarState::new(content_length).position(scroll);
    let paragraph = Paragraph::new(lines)
        .scroll((scroll as u16, 0))
        .block(title_block);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    frame.render_widget(paragraph, rect);
    frame.render_stateful_widget(
        scrollbar,
        rect.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state,
    );
}

fn render_pack_list(app: &mut App, frame: &mut Frame, outer_layout: Rect) {
    let filtered_packs = app.packs.get_pack_list().filtered_items();

    let title_block = Block::default()
        .title(format!("packs ({})", filtered_packs.len()))
        .borders(Borders::ALL);
    frame.render_widget(title_block, outer_layout);

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(75)])
        .margin(1)
        .split(outer_layout);

    let filter_title_block = Block::default().borders(Borders::BOTTOM);
    frame.render_widget(filter_title_block, inner_layout[0]);

    match app.menu_context.active_focus {
        ActiveFocus::Filter(ref mut textarea) => {
            textarea.set_cursor_line_style(Style::default().fg(Color::Cyan));
            textarea.set_block(Block::default().borders(Borders::ALL).fg(Color::Cyan).title("filter").title_alignment(Alignment::Right));
            textarea.set_placeholder_text("Filter by pack name");
            textarea.start_selection();
            textarea.move_cursor(CursorMove::WordForward);
            let widget = textarea.widget();
            frame.render_widget(widget, inner_layout[0]);
        }
        _ => {
            if let Some(ref mut pack_list) = app.packs.pack_list {
                let mut existing_filter = pack_list.filter.clone();
                if existing_filter.is_empty() {
                    existing_filter = "ctrl-f".to_string();
                }
                let line = Line::from(vec![
                    Span::styled(existing_filter, Style::default().fg(Color::White).bold()),
                ]);

                let paragraph = Paragraph::new(line)
                    .block(Block::default().borders(Borders::ALL).title("filter").title_alignment(Alignment::Right))
                    .alignment(Alignment::Left);
                frame.render_widget(paragraph, inner_layout[0]);
            }
        }
    }

    let list_items: Vec<ListItem> = filtered_packs
        .iter()
        .map(|pack| ListItem::new(pack.name.clone()))
        .collect();

    let items = List::new(list_items).highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    let pack_list = app.packs.get_pack_list();
    frame.render_stateful_widget(items, inner_layout[1], &mut pack_list.state);
}
