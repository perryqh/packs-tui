use std::rc::Rc;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use packs_client::pks::{PackDependentViolation, PathViolations, Pks};
use packs_client::pks_tree_node::PksTreeNode;
use ratatui::widgets::block::Title;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_tree_widget::{Tree, TreeItem};

use super::{Component, Frame};
use crate::components::helpers::active_violations::{
    ActiveViolations, CONTAINED_IN_SORTABLE, CONTAINED_OUT_SORTABLE, UNCONTAINED_IN_SORTABLE,
    UNCONTAINED_OUT_SORTABLE,
};
use crate::components::helpers::scroll_sortable::ScrollSortable;
use crate::components::helpers::stateful_tree::StatefulTree;
use crate::components::helpers::violations_display::ViolationsDisplay;
use crate::ui::shared::{build_root_level_vertical_layout, build_top_menu};
use crate::ui::style::Theme;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

pub struct Home<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    pks: Pks,
    pack_tree: StatefulTree<'a>,
    active_violations: ActiveViolations,
    active_panel: ActivePanel,
    scroll_sortable: ScrollSortable,
}
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActivePanel {
    Tree,
    Violations,
}

impl Default for ActivePanel {
    fn default() -> Self {
        Self::Tree
    }
}

struct InOutCount {
    active: bool,
    count: usize,
    fg_color: Color,
    menu_key: String,
}

impl Component for Home<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Down => {
                if self.active_panel == ActivePanel::Tree {
                    self.pack_tree.down();
                } else {
                    self.scroll_sortable.next_vertical_scroll();
                }
            }
            Action::Up => {
                if self.active_panel == ActivePanel::Tree {
                    self.pack_tree.up();
                } else {
                    self.scroll_sortable.previous_vertical_scroll();
                }
            }
            Action::Left => {
                if self.active_panel == ActivePanel::Tree {
                    self.pack_tree.left();
                } else {
                    // self.scroll_sortable.previous_horizontal_scroll();
                }
            }
            Action::Right => {
                if self.active_panel == ActivePanel::Tree {
                    self.pack_tree.right();
                } else {
                    // self.scroll_sortable.next_horizontal_scroll();
                }
            }
            Action::UncontainedOutViolations => {
                self.active_violations = ActiveViolations::Out;
            }
            Action::UncontainedInViolations => {
                self.active_violations = ActiveViolations::In;
            }
            Action::ContainedOutViolations => {
                self.active_violations = ActiveViolations::ContainedOut;
            }
            Action::ContainedInViolations => {
                self.active_violations = ActiveViolations::ContainedIn;
            }
            Action::NextTab => {
                if self.active_panel == ActivePanel::Tree {
                    self.active_panel = ActivePanel::Violations;
                } else {
                    self.scroll_sortable.next_focus_column();
                }
            }
            Action::Escape => {
                self.active_panel = ActivePanel::Tree;
            }
            Action::SortAscending => {
                self.scroll_sortable.set_sort_column_to_active_column();
                self.scroll_sortable.sort_ascending();
            }
            Action::SortDescending => {
                self.scroll_sortable.set_sort_column_to_active_column();
                self.scroll_sortable.sort_descending();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = build_root_level_vertical_layout(f);
        let top_menu = build_top_menu();
        f.render_widget(top_menu, chunks[0]);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(33), Constraint::Percentage(67)])
            .split(chunks[1]);
        self.draw_tree(f, outer_layout[0])?;

        let context_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(5), Constraint::Min(10)])
            .split(outer_layout[1]);
        self.render_in_out_violations(f, context_layout[0])?;
        self.render_violation_details(f, context_layout[1])?;
        Ok(())
    }
}

impl<'a> Home<'a> {
    pub fn new(mut pks: Pks) -> Self {
        let pks_tree_data = pks.get_pks_tree_data();
        let tree_items = build_tree_items(Rc::clone(&pks_tree_data));
        let pack_tree = StatefulTree::with_items(tree_items);
        let scroll_sortable = ScrollSortable::default();
        Self {
            pks,
            command_tx: None,
            config: Config::default(),
            pack_tree,
            active_violations: ActiveViolations::default(),
            active_panel: ActivePanel::default(),
            scroll_sortable,
        }
    }

    fn draw_tree(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let title_block = Block::default()
            .title(format!("packs ({})", self.pks.get_num_packs()))
            .borders(Borders::ALL)
            .border_style(match self.active_panel {
                ActivePanel::Tree => Style::default().fg(Color::Yellow),
                ActivePanel::Violations => Style::default(),
            })
            .title(match self.active_panel {
                ActivePanel::Tree => Title::default(),
                ActivePanel::Violations => Title::from("(esc)").alignment(Alignment::Right),
            });

        let items = Tree::new(self.pack_tree.items.clone())
            .expect("all item identifiers are unique")
            .block(title_block)
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let state = &mut self.pack_tree.state;
        f.render_stateful_widget(items, area, state);

        // TODO: bring this in from "saved preferences"
        if state.get_all_opened().is_empty() {
            state.select_first(&self.pack_tree.items);
            [
                vec![String::from("."), String::from("./packs")],
                vec![String::from("."), String::from("./packs")],
                vec![String::from(".")],
            ]
            .iter()
            .for_each(|path| {
                state.open(path.to_vec());
            });
        }
        Ok(())
    }

    fn selected_path_violations(&mut self) -> Option<Arc<PathViolations>> {
        let selected_paths = self.pack_tree.state.selected();
        if selected_paths.is_empty() {
            return None;
        }
        let show_path = selected_paths.last().unwrap();
        self.pks.get_path_violations_for_path(show_path)
    }

    fn render_violation_details(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let violations = self.selected_path_violations();
        if violations.is_none() {
            return Ok(());
        }
        let mut violations_display = ViolationsDisplay::new(
            &mut self.active_violations,
            violations.unwrap(),
            &self.active_panel,
            &mut self.scroll_sortable,
        );
        violations_display.render(f, area)?;

        Ok(())
    }

    pub(crate) fn render_in_out_violations(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let in_out_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let (
            mut uncontained_out_violations_count,
            mut uncontained_in_violations_count,
            mut contained_out_violations_count,
            mut contained_in_violations_count,
        ) = (0, 0, 0, 0);

        if let Some(violations) = self.selected_path_violations() {
            (
                uncontained_out_violations_count,
                uncontained_in_violations_count,
                contained_out_violations_count,
                contained_in_violations_count,
            ) = (
                violations.uncontained_out_violations_count,
                violations.uncontained_in_violations_count,
                violations.contained_out_violations_count,
                violations.contained_in_violations_count,
            );
        }
        let mut uncontained_out_count = InOutCount {
            active: false,
            count: uncontained_out_violations_count,
            fg_color: Color::Red,
            menu_key: "[0]".to_string(),
        };
        let mut uncontained_in_count = InOutCount {
            active: false,
            count: uncontained_in_violations_count,
            fg_color: Color::Yellow,
            menu_key: "[1]".to_string(),
        };
        let mut contained_out_count = InOutCount {
            active: false,
            count: contained_out_violations_count,
            fg_color: Color::LightBlue,
            menu_key: "[2]".to_string(),
        };
        let mut contained_in_count = InOutCount {
            active: false,
            count: contained_in_violations_count,
            fg_color: Color::LightCyan,
            menu_key: "[3]".to_string(),
        };
        match self.active_violations {
            ActiveViolations::Out => {
                uncontained_out_count.active = true;
            }
            ActiveViolations::In => {
                uncontained_in_count.active = true;
            }
            ActiveViolations::ContainedOut => {
                contained_out_count.active = true;
            }
            ActiveViolations::ContainedIn => {
                contained_in_count.active = true;
            }
        }

        let uncontained_table =
            build_in_out_table("Uncontained", &uncontained_out_count, &uncontained_in_count);
        let contained_table =
            build_in_out_table("Contained", &contained_out_count, &contained_in_count);

        f.render_widget(uncontained_table, in_out_layout[0]);
        f.render_widget(contained_table, in_out_layout[1]);
        Ok(())
    }
}

// https://github.com/EdJoPaTo/tui-rs-tree-widget/blob/main/examples/example.rs
fn build_tree_items<'a>(pks_tree_data: Rc<Vec<PksTreeNode>>) -> Vec<TreeItem<'a, String>> {
    pks_tree_data
        .iter()
        .map(map_tree_node_to_tree_item)
        .collect()
}

fn map_tree_node_to_tree_item<'a>(tree_node: &PksTreeNode) -> TreeItem<'a, String> {
    let tree_items = tree_node
        .children
        .as_ref()
        .map_or_else(Vec::new, |children| {
            children.iter().map(map_tree_node_to_tree_item).collect()
        });

    let styled_node_name = Span::styled(
        tree_node.node_name.clone(),
        Style::new()
            .fg(Color::Green)
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let styled_out_violation_count = Span::styled(
        format!(" ({})", tree_node.out_violation_count),
        Style::new()
            .fg(Color::Rgb(109, 0, 0))
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    let styled_in_violation_count = Span::styled(
        format!(" ({})", tree_node.in_violation_count),
        Style::new()
            .fg(Color::Rgb(139, 70, 0))
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    let line = Line::from(vec![
        styled_node_name,
        styled_out_violation_count,
        styled_in_violation_count,
    ]);

    let text: Text = Text::from(vec![line]);
    TreeItem::new(tree_node.path.clone(), text, tree_items).expect("Failed to create tree item")
}

fn build_in_out_table<'a>(
    title: &'a str,
    out_count: &'a InOutCount,
    in_count: &'a InOutCount,
) -> Table<'a> {
    let header = |title: &str, count: &InOutCount| {
        let (fg, bg) = if count.active {
            (Color::Black, count.fg_color)
        } else {
            (Color::White, Color::Black)
        };
        let title_span = Span::styled(title.to_string(), Style::default().fg(Color::White));
        let key_span = Span::styled(
            count.menu_key.clone(),
            Style::default()
                .fg(fg)
                .bg(bg)
                .add_modifier(Modifier::UNDERLINED),
        );
        Line::from(vec![title_span, key_span]).alignment(Alignment::Center)
    };

    let header_cells = vec![header("Out", out_count), header("In", in_count)];
    let header = Row::new(header_cells).bold().height(1);
    let build_cell = |count: &InOutCount| {
        Cell::from(
            Line::from(vec![Span::styled(
                count.count.to_string(),
                Style::default().fg(count.fg_color),
            )])
            .alignment(Alignment::Center),
        )
    };
    let row = vec![build_cell(out_count), build_cell(in_count)];
    let table = Table::new(
        vec![Row::new(row).height(2)],
        vec![Constraint::Length(14), Constraint::Length(14)],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));
    table
}
