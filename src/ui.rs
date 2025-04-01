use crate::spec_processor::{Method, Status};
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Row, Table};
use ratatui::{symbols, Frame};

pub fn render_table(model: &mut crate::AppModel, area: Rect, frame: &mut Frame) {
    let header = Row::new(vec!["    Summary", "Path", "Methods"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let rows = model.table_items.iter().map(|data| {
        let mut description = data.description.clone();
        if description.is_empty() {
            description = data
                .methods
                .iter()
                .map(|method| method.description.as_str())
                .collect::<Vec<&str>>()
                .join("/");
        }

        let description_selection = match data.status {
            Status::Unselected => format!("    {}", description),
            Status::Selected => format!(" ✂️ {}", description),
        };

        let row_style = if data.status == Status::Selected {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(vec![
            description_selection,
            data.path.clone(),
            data.methods
                .iter()
                .map(|method| method.method.as_str().to_uppercase())
                .collect::<Vec<String>>()
                .join(" "),
        ])
        .height(1)
        .style(row_style)
    });

    let table = Table::new(
        rows,
        [Constraint::Min(20), Constraint::Min(20), Constraint::Min(1)],
    )
    .header(header)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED | Modifier::ITALIC))
    .block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(format!(
                " {} endpoints for {}",
                model.table_items.len(),
                model.infile
            ))
            .title_alignment(Alignment::Center),
    );

    // Store the table area for pagination
    model.table_area = Some(area);

    frame.render_stateful_widget(table, area, &mut model.table_state);
}

pub fn render_detail(model: &crate::AppModel, area: Rect, frame: &mut Frame) {
    let selected_item = &model.table_items[model.table_state.selected().unwrap()];
    let mut description = selected_item.description.clone();
    if description.is_empty() {
        description = selected_item
            .methods
            .iter()
            .map(|method| method.description.as_str())
            .collect::<Vec<&str>>()
            .join("/");
    }

    let mut detail_lines: Vec<Line> = Vec::new();
    detail_lines.push(Line::from(description));
    detail_lines.push(Line::from("".to_string()));
    detail_lines.push(
        Line::from(selected_item.path.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
    );
    for method in selected_item.methods.iter() {
        detail_lines.push(styled_method(method));
    }

    let mut refs_lines: Vec<Line> = Vec::new();
    for reference in selected_item.refs.iter() {
        refs_lines.push(Line::from(format!("- {}", reference)));
    }
    if !refs_lines.is_empty() {
        detail_lines.push(Line::from("".to_string()));
        detail_lines.push(Line::from("Component schemas:".to_string()));
        detail_lines.extend(refs_lines);
    }

    let collapsed_top_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        ..symbols::border::PLAIN
    };

    let shortcuts = crate::shortcuts::Shortcuts::from(vec![
        ("q", "quit"),
        ("space", "✂️snip"),
        ("w", "write and quit"),
        ("↑", "move up"),
        ("↓", "move down"),
    ]);

    let selected_item_count = model
        .table_items
        .iter()
        .filter(|item| item.status == Status::Selected)
        .count();

    let detail = Paragraph::new(Text::from(detail_lines)).block(
        Block::default()
            .border_set(collapsed_top_border_set)
            .borders(Borders::ALL)
            .title(if selected_item_count > 0 {
                Line::from(vec![
                    " ".into(),
                    selected_item_count.to_string().bold().green().into(),
                    " endpoints selected ".into(),
                ])
            } else {
                Line::from("")
            })
            .title_bottom(shortcuts.as_line())
            .title_alignment(Alignment::Right)
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(detail, area);
}

fn styled_method(method: &Method) -> Line {
    let method_str = method.method.to_uppercase();
    let padded_method = format!("{:<6}", method_str);
    let the_method = Span::from(padded_method);

    let method_style = match method_str.as_str() {
        "GET" => Style::default().fg(Color::Blue),
        "PATCH" => Style::default().fg(Color::Yellow),
        "POST" => Style::default().fg(Color::Green),
        "PUT" => Style::default().fg(Color::Magenta),
        "DELETE" => Style::default().fg(Color::Red),
        _ => Style::default().add_modifier(Modifier::ITALIC),
    };

    Line::from(vec![
        the_method.style(method_style.add_modifier(Modifier::BOLD)),
        Span::from(" "),
        Span::from(method.description.clone()),
    ])
}
