use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};
use std::{collections::BTreeMap, iter::FromIterator};
use zellij_tile::prelude::{actions::Action, *};

mod renderer;

use crate::renderer::draw_to_string;

#[derive(Default)]
struct State {
    mode: InputMode,
    keybinds: Vec<(InputMode, Vec<(Key, Vec<Action>)>)>,
    userspace_configuration: BTreeMap<String, String>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;
        request_permission(&[PermissionType::ReadApplicationState]);
        subscribe(&[EventType::ModeUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        if let Event::ModeUpdate(mode_info) = event {
            self.keybinds = mode_info.keybinds;
            self.mode = mode_info.mode;

            should_render = true;
        }

        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        if rows == 0 || cols == 0 {
            return;
        }
        let width = cols as u16 - 1;
        let height = rows as u16 - 1;

        let mut mode_keybinds: Vec<(Key, Vec<Action>)> = Vec::new();
        let mut shared_keybinds: Vec<(Key, Vec<Action>)> = Vec::new();

        for (mode, keybinds) in &self.keybinds {
            for (key, actions) in keybinds {
                if *mode == self.mode {
                    let is_shared = self
                        .keybinds
                        .iter()
                        .filter(|(other_mode, other_keybinds)| {
                            *other_mode != self.mode
                                && other_keybinds.iter().any(|(other_key, other_actions)| {
                                    other_key == key && other_actions == actions
                                })
                        })
                        .count()
                        > 1;

                    if is_shared {
                        shared_keybinds.push((key.clone(), actions.clone()));
                    } else {
                        mode_keybinds.push((key.clone(), actions.clone()));
                    }
                }
            }
        }

        fn compare(a: &(Key, Vec<Action>), b: &(Key, Vec<Action>)) -> std::cmp::Ordering {
            let action_string = |a: &Vec<Action>| {
                a.iter()
                    .map(|action| format!("{:?}", action))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            match action_string(&a.1).cmp(&action_string(&b.1)) {
                std::cmp::Ordering::Equal => a.0.cmp(&b.0),
                a => a,
            }
        }

        mode_keybinds.sort_by(compare);
        shared_keybinds.sort_by(compare);

        fn keybinds_to_table(keybinds: &Vec<(Key, Vec<Action>)>, name: String) -> Table<'_> {
            let (rows, widths) = sliding_window(keybinds)
                .map(|(prev, row, next)| {
                    let mut cells = key_cells(&row.0, prev.map(|v| &v.0));
                    let mut actions = action_cells(&row.1, prev.map(|v| &v.1), next.map(|v| &v.1));
                    cells.append(&mut actions);
                    let (cells, widths) = cells.into_iter().unzip() as (Vec<_>, Vec<_>);
                    (Row::new(cells), widths)
                })
                .unzip() as (Vec<_>, Vec<_>);

            let col_widths = calculate_column_widths(&widths);

            let constraints: Vec<Constraint> = col_widths
                .iter()
                .map(|&w| Constraint::Length(w as u16))
                .collect();

            let table = Table::new(rows, &constraints)
                .block(
                    Block::default()
                        .borders(Borders::RIGHT)
                        .title(name)
                        .title_style(Style::new().fg(Color::Yellow))
                        .border_style(Style::new().fg(Color::DarkGray)),
                )
                .column_spacing(1)
                .highlight_style(Style::default().bg(Color::Blue));
            table
        }

        let mode_table = keybinds_to_table(&mode_keybinds, format!("{:?}", self.mode));
        let shared_table = keybinds_to_table(&shared_keybinds, "Shared".to_string());

        let rendered = draw_to_string(width, height, |f| {
            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.area());

            f.render_widget(mode_table, chunks[0]);
            f.render_widget(shared_table, chunks[1]);
        });

        print!("{}", rendered.unwrap());
    }
}

fn calculate_column_widths<T>(data: T) -> Vec<usize>
where
    T: AsRef<[Vec<usize>]>,
{
    data.as_ref().iter().fold(vec![], |mut acc, row| {
        for (i, cell) in row.iter().enumerate() {
            let len = *cell;
            if i >= acc.len() {
                acc.push(len);
            } else {
                if len > acc[i] {
                    acc[i] = len;
                }
            }
        }
        acc
    })
}

fn key_to_parts(key: &Key) -> [String; 3] {
    match key {
        Key::Ctrl(c) => ["Ctrl".to_string(), "+".to_string(), c.to_string()],

        Key::Alt(c) => ["Alt".to_string(), "+".to_string(), c.to_string()],

        Key::Char(c) => {
            let result = match c {
                '\n' => "Enter".to_string(),
                ' ' => "Space".to_string(),
                _ => c.to_string(),
            };
            ["".to_string(), "".to_string(), result]
        }

        Key::Left | Key::Right | Key::Up | Key::Down => {
            ["".to_string(), "".to_string(), format!("{:?}", key)]
        }

        _ => ["".to_string(), "".to_string(), format!("{:?}", key)],
    }
}

fn key_cells<'a>(key: &Key, prev: Option<&Key>) -> Vec<(Cell<'a>, usize)> {
    let cells = key_to_parts(key);
    let prev_cells = prev.map(key_to_parts);
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let prev_matches = if let Some(prev_cells) = &prev_cells {
                i == 0 && prev_cells.len() > i && prev_cells[i] == *cell
            } else {
                false
            };
            let style = if i == 1 || i == 0 && prev_matches {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            (Cell::from(Span::styled(cell.clone(), style)), cell.len())
        })
        .collect()
}

#[derive(Eq, PartialEq)]
enum ActionParts {
    Symbol(String),
    Syntax(String),
}

fn actions_to_parts<'a>(actions: &Vec<Action>) -> Vec<ActionParts> {
    fn unpack(value: &debug_parser::Value, parts: &mut Vec<ActionParts>) {
        if let Some(name) = value.name.clone() {
            parts.push(ActionParts::Symbol(name));
        }
        match &value.kind {
            debug_parser::ValueKind::Set(set) => {
                for (_i, _value) in set.values.iter().enumerate() {}
            }
            debug_parser::ValueKind::Map(map) => {
                for (i, value) in map.values.iter().enumerate() {
                    parts.push(ActionParts::Symbol(value.key.clone()));
                    parts.push(ActionParts::Syntax(": ".to_string()));
                    unpack(&value.value, parts);
                    if i != map.values.len() - 1 {
                        parts.push(ActionParts::Syntax(", ".to_string()));
                    }
                }
            }
            debug_parser::ValueKind::List(list) => {
                parts.push(ActionParts::Syntax("[".to_string()));
                for (i, value) in list.values.iter().enumerate() {
                    unpack(value, parts);
                    if i != list.values.len() - 1 {
                        parts.push(ActionParts::Syntax(", ".to_string()));
                    }
                }
                parts.push(ActionParts::Syntax("]".to_string()));
            }
            debug_parser::ValueKind::Tuple(tuple) => {
                parts.push(ActionParts::Syntax("(".to_string()));
                for (i, value) in tuple.values.iter().enumerate() {
                    unpack(value, parts);
                    if i != tuple.values.len() - 1 {
                        parts.push(ActionParts::Syntax(", ".to_string()));
                    }
                }
                parts.push(ActionParts::Syntax(")".to_string()));
            }
            debug_parser::ValueKind::Term(str) => parts.push(ActionParts::Symbol(str.to_string())),
        };
    }
    use debug_parser::parse;
    let val = parse(format!("{:?}", actions).as_str());
    let mut parts = Vec::new();
    unpack(&val, &mut parts);
    // Drop the outer array brackets
    parts.pop();
    parts.remove(0);

    parts
}

fn action_parts_to_line<'a>(parts: &Vec<ActionParts>, prev_parts: &Vec<ActionParts>) -> Line<'a> {
    let mut styled_parts = Vec::new();
    let mut differing = false;

    for (i, part) in parts.iter().enumerate() {
        if !differing {
            if let Some(prev_part) = prev_parts.get(i) {
                if part == prev_part {
                    let styled = match part {
                        ActionParts::Symbol(s) | ActionParts::Syntax(s) => {
                            Span::styled(s.clone(), Style::default().fg(Color::DarkGray))
                        }
                    };
                    styled_parts.push(styled);
                    continue;
                }
            }
            differing = true;
        }

        let styled = match part {
            ActionParts::Symbol(s) => Span::styled(s.clone(), Style::default().fg(Color::White)),
            ActionParts::Syntax(s) => Span::styled(s.clone(), Style::default().fg(Color::DarkGray)),
        };
        styled_parts.push(styled);
    }

    Line::from_iter(styled_parts)
}

fn action_cells<'a>(
    actions: &Vec<Action>,
    prev: Option<&Vec<Action>>,
    next: Option<&Vec<Action>>,
) -> Vec<(Cell<'a>, usize)> {
    let prev_match = if let Some(prev) = prev {
        prev.eq(actions)
    } else {
        false
    };
    let prev_parts = if let Some(prev) = prev {
        actions_to_parts(prev)
    } else {
        vec![]
    };
    let next_match = if let Some(next) = next {
        next.eq(actions)
    } else {
        false
    };
    let symbol = match (prev_match, next_match) {
        (false, true) => "┳".to_string(),
        (true, true) => "┫".to_string(),
        (true, false) => "┛".to_string(),
        (false, false) => "━".to_string(),
    };
    let text = if prev_match {
        Line::raw("")
    } else {
        action_parts_to_line(&actions_to_parts(actions), &prev_parts)
    };
    let len = text.iter().map(|t| t.width()).sum();
    vec![(Cell::from(symbol), 1), (Cell::from(text), len)]
}

fn sliding_window<'a, T>(
    data: &'a [T],
) -> impl Iterator<Item = (Option<&'a T>, &'a T, Option<&'a T>)> {
    (0..data.len()).map(move |i| {
        let prev = if i == 0 { None } else { Some(&data[i - 1]) };
        let row = &data[i];
        let next = if i == data.len() - 1 {
            None
        } else {
            Some(&data[i + 1])
        };
        (prev, row, next)
    })
}
