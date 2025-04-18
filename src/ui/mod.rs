pub mod color;
pub mod widget;

use crate::spec_processor::{Method, Status};
use crate::ui::color::gradient_color;
use crate::ui::widget::Shortcuts;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Padding, Paragraph, Row, Scrollbar, ScrollbarState, Table,
};
use ratatui::{symbols, Frame};
use widget::Shortcut;

// Helper function to calculate visible rows in the table
fn calculate_visible_table_rows(model: &crate::AppModel) -> usize {
    // Each row is 1 line high, header is 1 line, borders are 2 lines
    let total_rows = model.table_items.len();
    let visible_rows = model
        .table_area
        .map(|area| area.height.saturating_sub(3) as usize)
        .unwrap_or(1);
    visible_rows.min(total_rows)
}

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

    // Get the currently selected index for calculating distance
    let selected_idx = model.table_state.selected().unwrap_or(0);

    let rows = model.table_items.iter().enumerate().map(|(idx, data)| {
        // Use a reference to the description to avoid cloning
        let description = if data.description.is_empty() {
            data.methods
                .iter()
                .map(|method| method.description.as_str())
                .collect::<Vec<&str>>()
                .join("/")
        } else {
            data.description.to_string()
        };

        let description_selection = match data.status {
            Status::Unselected => format!("    {}", description),
            Status::Selected => format!(" ✂️ {}", description),
        };

        // Calculate distance from selected row to apply gradient
        let distance = if idx > selected_idx {
            idx - selected_idx
        } else {
            selected_idx - idx
        };

        // Determine if this row should be selected (green)
        let is_selected_item = data.status == Status::Selected;

        // Apply gradient styling based on distance and color support
        let row_style = gradient_color(
            distance,
            idx == selected_idx,
            is_selected_item,
            model.color_support,
            model.default_foreground_color,
            model.color_mode,
        );

        // Use references for path and methods to avoid cloning
        Row::new(vec![
            description_selection,
            data.path.to_string(),
            data.methods
                .iter()
                .map(|method| method.method.to_uppercase())
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
            .title_alignment(Alignment::Center)
            .style(model.default_style),
    );

    // Calculate scrollbar state
    let visible_rows = calculate_visible_table_rows(model);
    let total_rows = model.table_items.len();
    let mut scrollbar_state = ScrollbarState::new(total_rows)
        .position(selected_idx)
        .viewport_content_length(visible_rows);

    // Render the table
    frame.render_stateful_widget(table, area, &mut model.table_state);

    // Render the scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("█");

    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
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
    detail_lines.push(Line::from(selected_item.path.clone()).style(Style::default()));
    for method in selected_item.methods.iter() {
        detail_lines.push(styled_method_with_description(method, 6));
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

    let shortcuts = Shortcuts::new(vec![
        Shortcut::Pair("space", "✂️snip"),
        Shortcut::Pair("w", "write and quit"),
        Shortcut::Pair("/", "search"),
        Shortcut::Trio("▼", "move", "▲"),
        Shortcut::Pair("q", "quit"),
    ])
    .with_alignment(Alignment::Right)
    .with_label_style(model.default_style.add_modifier(Modifier::BOLD));

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
            .title_alignment(Alignment::Right)
            .title_bottom(shortcuts.as_line())
            .padding(Padding::new(1, 1, 0, 0))
            .style(model.default_style),
    );
    frame.render_widget(detail, area);
}

pub fn render_search(model: &mut crate::AppModel, area: Rect, frame: &mut Frame) {
    let collapsed_top_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        ..symbols::border::PLAIN
    };

    let shortcuts = Shortcuts::new(vec![
        Shortcut::Pair("🔍", "search"),
        Shortcut::Pair("Esc", "exit search"),
        Shortcut::Pair("Ctrl+U", "clear search"),
    ])
    .with_alignment(Alignment::Left)
    .with_label_style(model.default_style.add_modifier(Modifier::BOLD));

    let block = Block::default()
        .padding(Padding {
            left: 1,
            right: 0,
            top: 0,
            bottom: 0,
        })
        .border_set(collapsed_top_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(shortcuts.as_line())
        .style(model.default_style);

    let inner_area = block.inner(area);

    frame.render_widget(block, area);

    frame.render_widget(&model.search_state.text_input, inner_area);
}

fn styled_method_with_description(method: &Method, padding: usize) -> Line {
    Line::from(vec![
        colored_method(&method.method, padding).add_modifier(Modifier::BOLD),
        Span::from(" "),
        Span::from(method.description.clone()),
    ])
}

fn colored_method(method: &str, padding: usize) -> Span {
    let method_str = method.to_uppercase();
    let the_method: Span = if padding > 0 {
        Span::from(format!("{:<padding$}", method_str.clone()))
    } else {
        Span::from(method_str.clone())
    };

    let method_style = match method_str.as_str() {
        "GET" => Style::default().fg(Color::Blue),
        "PATCH" => Style::default().fg(Color::Yellow),
        "POST" => Style::default().fg(Color::Green),
        "PUT" => Style::default().fg(Color::Magenta),
        "DELETE" => Style::default().fg(Color::Red),
        "HEAD" => Style::default().fg(Color::Cyan),
        _ => Style::default().add_modifier(Modifier::ITALIC),
    };

    the_method.style(method_style)
}
