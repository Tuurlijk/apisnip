pub mod widget;

use crate::spec_processor::{Method, Status};
use crate::ui::widget::Shortcuts;
use crate::{rgb_to_indexed, Mode};
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Row, Table};
use ratatui::{symbols, Frame};
use supports_color::ColorLevel;

// Helper to calculate a gradient color based on distance from selected row
fn gradient_color(
    distance: usize,
    selected: bool,
    is_selected_item: bool,
    color_level: Option<ColorLevel>,
    default_foreground: (u8, u8, u8),
    color_mode: Mode,
) -> Style {
    // If this is the selected row, use reversed style
    if selected {
        return Style::default().add_modifier(Modifier::REVERSED | Modifier::ITALIC);
    }

    // If this is a selected item (✂️), use green/bold regardless of distance
    if is_selected_item {
        return Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
    }

    // For terminals with no color support, just return default style
    if color_level.is_none() {
        return Style::default();
    }

    // No effect for selected row and immediate neighbors
    if distance <= 0 {
        return Style::default();
    }

    // Maximum distance for gradient effect
    let max_distance = 20;

    let progress = distance as f32 / max_distance as f32;

    // Apply linear gradient based on terminal capabilities
    let foreground = default_foreground;

    // Calculate dimmed foreground color based on color mode
    let dimmed = calculate_dimmed_color(foreground, color_mode);

    // Calculate interpolated color with proper clamping based on color mode
    let color = interpolate_color(foreground, dimmed, progress, color_mode);

    // Create style with the calculated color
    match color_level {
        Some(level) if level.has_16m => {
            // For truecolor terminals, use RGB directly
            Style::default().fg(Color::Rgb(color.0, color.1, color.2))
        }
        Some(level) if level.has_256 => {
            // For 256-color terminals, convert to indexed color
            Style::default().fg(Color::Indexed(rgb_to_indexed(color.0, color.1, color.2)))
        }
        _ => {
            // For basic terminals, use simple dimming
            if progress > 0.5 {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            }
        }
    }
}

// Calculate dimmed foreground color based on color mode
fn calculate_dimmed_color(foreground: (u8, u8, u8), color_mode: Mode) -> (u8, u8, u8) {
    match color_mode {
        Mode::Dark => (
            (foreground.0 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.1 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.2 as f32 * 0.75).clamp(0.0, 255.0) as u8,
        ),
        Mode::Light => (
            (foreground.0 as f32 * 1.5).clamp(0.0, 255.0) as u8,
            (foreground.1 as f32 * 1.5).clamp(0.0, 255.0) as u8,
            (foreground.2 as f32 * 1.5).clamp(0.0, 255.0) as u8,
        ),
        _ => (
            (foreground.0 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.1 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.2 as f32 * 0.75).clamp(0.0, 255.0) as u8,
        ),
    }
}

// Interpolate between foreground and dimmed colors based on progress
fn interpolate_color(
    foreground: (u8, u8, u8),
    dimmed: (u8, u8, u8),
    progress: f32,
    color_mode: Mode,
) -> (u8, u8, u8) {
    let r = interpolate_component(foreground.0, dimmed.0, progress, color_mode);
    let g = interpolate_component(foreground.1, dimmed.1, progress, color_mode);
    let b = interpolate_component(foreground.2, dimmed.2, progress, color_mode);
    (r, g, b)
}

// Interpolate a single color component with proper clamping based on color mode
fn interpolate_component(fg: u8, dimmed: u8, progress: f32, color_mode: Mode) -> u8 {
    let value = fg as f32 + ((dimmed as f32 - fg as f32) * progress);

    // Clamp the value based on color mode
    let clamped = match color_mode {
        Mode::Dark => value.clamp(dimmed as f32, fg as f32),
        _ => value.clamp(fg as f32, dimmed as f32),
    };

    clamped as u8
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

    // let mode = match model.color_mode {
    //     Mode::Dark => "Dark",
    //     Mode::Light => "Light",
    //     Mode::Unspecified => "Unspecified",
    // };
    // let depth = match model.color_support {
    //     Some(level) => {
    //         if level.has_16m {
    //             "24-bit"
    //         } else if level.has_256 {
    //             "16-bit"
    //         } else {
    //             "8-bit"
    //         }
    //     }
    //     None => "No color support",
    // };

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

    let shortcuts = Shortcuts::new(vec![
        ("q", "quit"),
        ("space", "✂️snip"),
        ("w", "write and quit"),
        ("↑", "move up"),
        ("↓", "move down"),
        ("/", "search"),
        ("Esc", "exit search"),
        ("Home", "to top"),
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

    let block = Block::default()
        .border_set(collapsed_top_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(" 🔍 ")
        .style(model.default_style);

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
