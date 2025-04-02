pub mod widget;

use crate::spec_processor::{Method, Status};
use crate::ui::widget::Shortcuts;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Row, Table};
use ratatui::{symbols, Frame};

pub fn render_table(model: &mut crate::AppModel, area: Rect, frame: &mut Frame) {
    // Store the table area for pagination
    model.table_area = Some(area);

    // Handle the case when there are no items to display
    if model.table_items.is_empty() {
        let no_items = Paragraph::new("No items match your search.")
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::Rounded)
                    .title(format!(" 0 endpoints for {} ", model.infile))
                    .title_alignment(Alignment::Center),
            )
            .alignment(Alignment::Center);

        frame.render_widget(no_items, area);
        return;
    }

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
            .border_type(BorderType::Rounded)
            .title(format!(
                " {} endpoints for {} ",
                model.table_items.len(),
                model.infile
            ))
            .title_alignment(Alignment::Center),
    );

    frame.render_stateful_widget(table, area, &mut model.table_state);
}

pub fn render_detail(model: &crate::AppModel, area: Rect, frame: &mut Frame) {
    // Check if we have any items to display and a valid selection
    if model.table_items.is_empty() || model.table_state.selected().is_none() {
        // Render an empty detail view with a message
        let detail = Paragraph::new("No items selected or search results are empty.")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(detail, area);
        return;
    }

    let selected_idx = model.table_state.selected().unwrap();

    // Ensure the selected index is valid
    if selected_idx >= model.table_items.len() {
        // Render an empty detail view with error message
        let detail = Paragraph::new("Invalid selection index.")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(detail, area);
        return;
    }

    let selected_item = &model.table_items[selected_idx];
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

    let mut refs_lines: Vec<String> = Vec::new();
    for reference in selected_item.refs.iter() {
        refs_lines.push(reference.to_string());
    }
    if !refs_lines.is_empty() {
        detail_lines.push(Line::from("".to_string()));
        detail_lines.push(Line::from(format!(
            "Component schemas: {}",
            refs_lines.join(", ")
        )));
    }

    let collapsed_top_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        bottom_right: symbols::line::ROUNDED_BOTTOM_RIGHT,
        bottom_left: symbols::line::ROUNDED_BOTTOM_LEFT,
        ..symbols::border::PLAIN
    };

    let shortcuts = Shortcuts::from(vec![
        ("q", "quit"),
        ("space", "✂️snip"),
        ("w", "write and quit"),
        ("↑", "move up"),
        ("↓", "move down"),
        ("/", "search"),
        ("Esc", "exit search"),
        ("Home", "to top"),
    ])
    .with_alignment(Alignment::Right);

    let selected_item_count = model
        .table_items
        .iter()
        .filter(|item| item.status == Status::Selected)
        .count();

    let detail = Paragraph::new(Text::from(detail_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(collapsed_top_border_set)
            .title(if selected_item_count > 0 {
                Line::from(vec![
                    " ".into(),
                    selected_item_count.to_string().bold().green(),
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

pub fn render_search(model: &mut crate::AppModel, area: Rect, frame: &mut Frame) {
    let collapsed_top_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        ..symbols::border::PLAIN
    };

    let block = Block::default()
        .border_set(collapsed_top_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(" 🔍 ");

    model.search_state.text_input.set_block(block);
    frame.render_widget(&model.search_state.text_input, area);
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
        "HEAD" => Style::default().fg(Color::Cyan),
        _ => Style::default().add_modifier(Modifier::ITALIC),
    };

    Line::from(vec![
        the_method.style(method_style.add_modifier(Modifier::BOLD)),
        Span::from(" "),
        Span::from(method.description.clone()),
    ])
}
