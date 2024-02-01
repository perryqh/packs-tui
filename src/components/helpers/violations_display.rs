use crate::components::helpers::active_violations::ActiveViolations;
use crate::components::helpers::scroll_sortable::ScrollSortable;
use crate::components::home::ActivePanel;
use crate::tui::Frame;
use color_eyre::owo_colors::OwoColorize;
use log::info;
use packs_client::pks::{PackDependentViolation, PathViolations};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::{Line, Span, Style, Stylize};
use ratatui::style::{Color, Modifier};
use ratatui::widgets::block::Title;
use ratatui::widgets::{
    Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    TableState, Wrap,
};
use std::collections::HashMap;
use std::iter::Map;
use std::sync::Arc;

const UNCONTAINED_OUT_COLOR: Color = Color::Red;
const UNCONTAINED_IN_COLOR: Color = Color::Yellow;
const CONTAINED_OUT_COLOR: Color = Color::LightBlue;
const CONTAINED_IN_COLOR: Color = Color::LightCyan;
pub struct ViolationsDisplay<'a> {
    pub active_violations: &'a mut ActiveViolations,
    pub scroll_sortable: &'a mut ScrollSortable,
    pub active_panel: &'a ActivePanel,
    pub selected_path_violations: Arc<PathViolations>,
    relevant_violations: Option<Vec<Arc<PackDependentViolation>>>,
}

struct ViolationSpecifics<'a> {
    violations: Vec<Arc<PackDependentViolation>>,
    table_title: &'a str,
    color: Color,
}

pub const VIOLATION_HEADER_ABBR_TITLES: [&str; 7] =
    ["def pack", "ref pack", "priv", "arch", "dep", "fvis", "vis"];
pub const VIOLATION_HEADER_FULL_TITLES: [&str; 7] = [
    "def pack",
    "ref pack",
    "privacy",
    "architecture",
    "dependency",
    "folder visibility",
    "visibility",
];
pub const MIN_HEIGHT_FOR_CONSTANTS: u16 = 20;

impl<'a> ViolationsDisplay<'a> {
    pub fn new(
        active_violations: &'a mut ActiveViolations,
        selected_path_violations: Arc<PathViolations>,
        active_panel: &'a ActivePanel,
        scroll_sortable: &'a mut ScrollSortable,
    ) -> Self {
        Self {
            active_violations,
            active_panel,
            selected_path_violations,
            relevant_violations: None,
            scroll_sortable,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let show_constants = area.height > MIN_HEIGHT_FOR_CONSTANTS;
        let layout_constraints = if show_constants {
            vec![Constraint::Percentage(80), Constraint::Percentage(20)]
        } else {
            vec![Constraint::Percentage(100)]
        };
        let violations_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(layout_constraints)
            .split(area);
        let area = violations_area[0];

        let violation_specifics = self.get_violation_specifics();
        let (constraint_widths, cols_width) = self.get_constraint_widths(&violation_specifics);
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let header_cells: Vec<Line> = VIOLATION_HEADER_ABBR_TITLES
            .iter()
            .enumerate()
            .map(|(index, header_title)| {
                let mut header_title = header_title.to_string();
                if index == self.scroll_sortable.sort_column() {
                    header_title = format!(
                        "{} {}",
                        if self.scroll_sortable.is_sort_ascending() {
                            "▼"
                        } else {
                            "▲"
                        },
                        header_title
                    )
                }
                if index == self.scroll_sortable.focused_column() {
                    let title_span = Span::styled(
                        header_title.to_string(),
                        Style::default()
                            .bg(violation_specifics.color)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    );
                    Line::from(vec![title_span]).alignment(Alignment::Center)
                } else {
                    let title_span =
                        Span::styled(header_title.to_string(), Style::default().fg(Color::White));
                    Line::from(vec![title_span]).alignment(Alignment::Center)
                }
            })
            .collect();

        let header = Row::new(header_cells).bold().height(1);
        let rows = violation_specifics.violations.iter().map(|violation| {
            let height = 1;
            let mut cells = vec![];
            cells.push(Cell::from(violation.defining_pack_name.clone()));
            cells.push(Cell::from(violation.referencing_pack_name.clone()));
            for key in VIOLATION_HEADER_FULL_TITLES.iter().skip(2) {
                let count = violation.count_for_violation_type(key);
                cells.push(Cell::from(count.to_string()));
            }
            Row::new(cells).height(height as u16)
        });

        let rows_len = &rows.len();

        let border_style = match self.active_panel {
            ActivePanel::Tree => Style::default().fg(Color::White),
            ActivePanel::Violations => Style::default().fg(violation_specifics.color),
        };
        let table = Table::new(rows, constraint_widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Violations ({})", rows_len))
                    .title(Title::from(violation_specifics.table_title).alignment(Alignment::Right))
                    .border_style(border_style),
            )
            .highlight_style(selected_style)
            .highlight_symbol(">> ");

        let mut vertical_scroll = self.scroll_sortable.vertical_scroll();
        if vertical_scroll >= *rows_len && !&violation_specifics.violations.is_empty() {
            vertical_scroll = *rows_len - 1;
            self.scroll_sortable.set_vertical_scroll(vertical_scroll);
        }
        let mut table_state =
            TableState::default().with_selected(Some(self.scroll_sortable.vertical_scroll()));

        f.render_stateful_widget(table, area, &mut table_state);

        let vertical_scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut vertical_scrollbar_state =
            ScrollbarState::new(*rows_len).position(self.scroll_sortable.vertical_scroll());

        f.render_stateful_widget(
            vertical_scrollbar,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
            &mut vertical_scrollbar_state,
        );

        // let horizontal_scrollbar = Scrollbar::default()
        //     .orientation(ScrollbarOrientation::HorizontalBottom)
        //     .begin_symbol(Some("←"))
        //     .end_symbol(Some("→"));
        // let mut horizontal_scrollbar_state =
        //     ScrollbarState::new(cols_width).position(self.active_violations.horizontal_scroll());
        //
        // f.render_stateful_widget(
        //     horizontal_scrollbar,
        //     area.inner(&Margin {
        //         vertical: 0,
        //         horizontal: 1,
        //     }),
        //     &mut horizontal_scrollbar_state,
        // );

        if show_constants {
            if let Some(constants_paragraph) =
                self.build_constants(&violation_specifics, self.scroll_sortable.vertical_scroll())
            {
                f.render_widget(constants_paragraph, violations_area[1]);
            }
        }
        Ok(())
    }

    fn build_constants(
        &self,
        violation_specifics: &ViolationSpecifics,
        selected_index: usize,
    ) -> Option<Paragraph<'a>> {
        if let Some(constant_counts) = violation_specifics.constant_counts_for_index(selected_index)
        {
            let spans: Vec<Span> = constant_counts
                .iter()
                .flat_map(|(pack, count)| {
                    vec![
                        Span::styled(
                            format!("({})", count),
                            Style::default().light_cyan().italic(),
                        ),
                        Span::styled(format!("{} ", pack), Style::default().gray()),
                    ]
                })
                .collect();
            let line = Line::from(spans);
            let paragraph = Paragraph::new(line)
                .block(Block::new().title("constant counts").borders(Borders::ALL))
                .style(Style::new().gray().on_black())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            Some(paragraph)
        } else {
            None
        }
    }

    fn get_violation_specifics(&mut self) -> ViolationSpecifics<'a> {
        let mut specifics = match self.active_violations {
            ActiveViolations::Out => ViolationSpecifics {
                violations: self
                    .selected_path_violations
                    .uncontained_out_violations
                    .clone(),
                table_title: "uncontained outgoing violations",
                color: UNCONTAINED_OUT_COLOR,
            },
            ActiveViolations::In => ViolationSpecifics {
                violations: self
                    .selected_path_violations
                    .uncontained_in_violations
                    .clone(),
                table_title: "uncontained incoming violations",
                color: UNCONTAINED_IN_COLOR,
            },
            ActiveViolations::ContainedOut => ViolationSpecifics {
                violations: self
                    .selected_path_violations
                    .contained_out_violations
                    .clone(),
                table_title: "internal outgoing violations",
                color: CONTAINED_OUT_COLOR,
            },
            ActiveViolations::ContainedIn => ViolationSpecifics {
                violations: self
                    .selected_path_violations
                    .contained_in_violations
                    .clone(),
                table_title: "internal incoming violations",
                color: CONTAINED_IN_COLOR,
            },
        };
        match self.scroll_sortable.sort_column() {
            0 => specifics
                .violations
                .sort_by(|a, b| a.defining_pack_name.cmp(&b.defining_pack_name)),
            1 => specifics
                .violations
                .sort_by(|a, b| a.referencing_pack_name.cmp(&b.referencing_pack_name)),
            2..=6 => specifics.violations.sort_by(|a, b| {
                a.count_for_violation_type(
                    VIOLATION_HEADER_FULL_TITLES[self.scroll_sortable.sort_column()],
                )
                .cmp(&b.count_for_violation_type(
                    VIOLATION_HEADER_FULL_TITLES[self.scroll_sortable.sort_column()],
                ))
                .then(a.defining_pack_name.cmp(&b.defining_pack_name))
            }),
            _ => {}
        }
        if !self.scroll_sortable.is_sort_ascending() {
            specifics.violations.reverse();
        }
        specifics
    }

    fn get_constraint_widths(
        &self,
        violation_specifics: &ViolationSpecifics,
    ) -> (Vec<Constraint>, usize) {
        let max_def_pack_name_len = violation_specifics
            .violations
            .iter()
            .map(|violation| violation.defining_pack_name.len())
            .max()
            .unwrap_or(0);
        let max_ref_pack_name_len = violation_specifics
            .violations
            .iter()
            .map(|violation| violation.referencing_pack_name.len())
            .max()
            .unwrap_or(0);
        let count_widths = VIOLATION_HEADER_ABBR_TITLES.iter().skip(2).map(|h| h.len());
        let count_widths_sum = &count_widths.clone().sum::<usize>();
        let cols_width = max_def_pack_name_len + max_ref_pack_name_len + count_widths_sum + 5 /* cushion */;
        let mut constraint_widths = vec![
            Constraint::Ratio(max_def_pack_name_len as u32, cols_width as u32),
            Constraint::Ratio(max_ref_pack_name_len as u32, cols_width as u32),
        ];
        count_widths.for_each(|w| constraint_widths.push(Constraint::Length(w as u16)));
        (constraint_widths, cols_width)
    }
}

impl<'a> ViolationSpecifics<'a> {
    fn constant_counts_for_index(&self, index: usize) -> Option<Vec<(String, usize)>> {
        if let Some(violation) = self.violations.get(index) {
            let mut constant_counts: Vec<(String, usize)> = violation
                .constant_counts
                .iter()
                .map(|(constant, count)| (constant.clone(), *count))
                .collect();

            constant_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            Some(constant_counts)
        } else {
            None
        }
    }
}
