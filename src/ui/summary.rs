use crate::app::{App, ContextMenuItem, CONSTANT_VIOLATION_COLUMNS};
use crate::ui::shared::{
    build_chunks, build_context_menu, build_top_menu, centered_rect, render_filter_textarea,
};
use ratatui::{prelude::*, widgets::*};

pub fn render_summary(app: &mut App, frame: &mut Frame) {
    let chunks = build_chunks(frame);
    let top_menu_tabs = build_top_menu(app);
    frame.render_widget(top_menu_tabs, chunks[0]);
    let menu_titles = vec!["Info".to_string(), "Constant Violations".to_string()];

    let context_menu_index = match app.menu_context.active_context_menu_item {
        ContextMenuItem::ConstantViolations(_) => {
            render_context_menu_constant_violations(app, frame, chunks[1]);
            1
        }
        _ => {
            let widths = [Constraint::Length(30), Constraint::Length(5)];
            let rows = app.packs.get_summary().into_iter().map(|(key, value)| {
                Row::new(vec![
                    Cell::from(key).style(Style::default().fg(Color::White)),
                    Cell::from(value).style(Style::default().fg(Color::LightCyan)),
                ])
                .bottom_margin(1)
            });
            let table = Table::new(rows, widths).column_spacing(3);

            frame.render_widget(table, centered_rect(frame.size(), 55, 55));
            0
        }
    };
    if let Some(context_menu_tabs) = build_context_menu(&menu_titles, context_menu_index) {
        frame.render_widget(context_menu_tabs, chunks[2])
    }
}

fn render_context_menu_constant_violations(app: &mut App, frame: &mut Frame, rect: Rect) {
    let context_menu_constant_violations = match app.menu_context.active_context_menu_item {
        ContextMenuItem::ConstantViolations(ref mut context_menu_constant_violations) => {
            context_menu_constant_violations
        }
        _ => panic!("expected ContextMenuItem::ConstantViolations"),
    };
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_titles = &context_menu_constant_violations.header_titles();
    let header_cells = header_titles
        .iter()
        .enumerate()
        .map(|(index, header_title)| {
            if context_menu_constant_violations.sort_column == index {
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
    let constant_summaries = app.packs.get_constant_violation_summaries();
    let violations = &mut constant_summaries.filtered_summaries();
    context_menu_constant_violations.sort_violations(violations);
    let mut scroll = context_menu_constant_violations.scroll;
    if scroll >= violations.len() && !violations.is_empty() {
        scroll = violations.len() - 1;
        app.menu_context.active_context_menu_item.set_scroll(scroll);
    }
    let mut table_state = TableState::default().with_selected(Some(scroll));
    let rows = violations.iter().map(|violation| {
        let height = 1;
        let mut cells = vec![];
        cells.push(Cell::from(violation.constant.clone()));
        cells.push(Cell::from(violation.count.to_string()));
        CONSTANT_VIOLATION_COLUMNS
            .iter()
            .skip(2)
            .take(5)
            .for_each(|key| {
                let count = violation.count_for_violation_type(key);
                cells.push(Cell::from(count.to_string()));
            });
        let total_referencing_packs: usize = violation.referencing_pack_count_length();
        cells.push(Cell::from(total_referencing_packs.to_string()));

        Row::new(cells).height(height as u16)
    });

    let mut widths = vec![Constraint::Percentage(50)];
    header_titles
        .iter()
        .skip(1)
        .for_each(|h| widths.push(Constraint::Length(h.len() as u16)));
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(selected_style)
        .highlight_symbol(">> ");

    let title_block = Block::default()
        .title(format!("constant violations ({})", violations.len()))
        .borders(Borders::ALL);
    frame.render_widget(title_block, rect);

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(75), Constraint::Length(3)])
        .margin(1)
        .split(rect);

    frame.render_stateful_widget(table, inner_layout[0], &mut table_state);
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(violations.len())
        .position(scroll);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    frame.render_stateful_widget(
        scrollbar,
        inner_layout[0].inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state,
    );
    let info_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(35), Constraint::Percentage(75)])
        .split(inner_layout[1]);
    render_filter_textarea(
        &mut app.menu_context.active_focus,
        frame,
        info_layout[0],
        &constant_summaries.filter,
    );

    if let Some(violation) = violations.get(scroll) {
        let mut pack_counts: Vec<(String, usize)> = violation
            .referencing_pack_counts
            .iter()
            .map(|(pack, count)| (pack.clone(), *count))
            .collect();
        pack_counts.sort_by(|a, b| b.1.cmp(&a.1));
        let spans: Vec<Span> = pack_counts
            .iter()
            .flat_map(|(pack, count)| {
                vec![Span::styled(format!("({})", count), Style::new().light_cyan().italic()),
                     Span::styled(format!("{} ", pack), Style::new().white()),]
            })
            .collect();
        let line = Line::from(spans);
        let paragraph = Paragraph::new(line)
            .block(Block::new().title("pack counts").borders(Borders::ALL))
            .style(Style::new().white().on_black())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, info_layout[1]);
    }
}
