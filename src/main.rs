use std::fs;
use std::io::stdout;
use std::time::Duration;

mod shortcuts;

use clap::Parser;
use color_eyre::eyre::{self, OptionExt};
use crossterm::ExecutableCommand;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
use itertools::Itertools;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Row, Table, TableState};
use ratatui::{Frame, symbols};
use serde_yaml::{self, Mapping, Value};

#[derive(Debug, Default)]
struct AppModel {
    table_items: Vec<Endpoint>,
    table_state: TableState,
    running_state: RunningState,
    table_area: Option<ratatui::layout::Rect>,
    infile: String,
    outfile: String,
    spec: Mapping,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
struct Endpoint {
    methods: Vec<Method>,
    path: String,
    description: String,
    refs: Vec<String>,
    status: Status,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Status {
    #[default]
    Unselected,
    Selected,
}

#[derive(Debug, Default)]
struct Method {
    method: String,
    description: String,
}

#[derive(PartialEq, Copy, Clone)]
enum Message {
    SelectNext,
    SelectPrevious,
    SelectRow(u16),
    ToggleSelectItemAndSelectNext,
    SelectNextPage,
    SelectPreviousPage,
    WriteAndQuit,
    Quit,
}

/// Trim an API surface down to size
#[derive(Parser)]
#[clap(version, about = about_str())]
pub struct Args {
    /// The name of the input file
    #[clap()]
    infile: String,

    /// The name of the output file
    #[clap(default_value = "apisnip.out.yaml")]
    outfile: String,

    /// Enable verbose output
    #[clap(short = 'v', long, action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

fn about_str() -> &'static str {
    // Fetch value from the environment variable
    let dynamic_value = env!("GIT_INFO").to_string();

    // Build the about string with the dynamic value
    let about_str = format!(
        r"                     _____     _ _____     _     
                    |  _  |___|_|   __|___|_|___ 
                    |     | . | |__   |   | | . |
                    |__|__|  _|_|_____|_|_|_|  _|
                          |_|               |_|  

                   Trim an API surface down to size
                    Coded with ♥️ by Michiel Roos
                        {}
",
        dynamic_value
    );

    // Leak the dynamic string to get a static reference
    Box::leak(about_str.into_boxed_str())
}

fn main() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    let args = Args::parse();
    stdout().execute(EnableMouseCapture)?;
    let input_yaml = fs::read_to_string(&args.infile)?;
    let spec: Mapping = serde_yaml::from_str(&input_yaml)?;
    let mut model = AppModel {
        infile: args.infile,
        outfile: args.outfile,
        spec: spec,
        ..Default::default()
    };
    model.table_items = fetch_endpoints_from_spec(&model.spec);

    // Select the first row if no row is selected
    if model.table_state.selected().is_none() {
        model.table_state.select_first();
    }

    let mut terminal = tui::init_terminal()?;
    while model.running_state != RunningState::Done {
        // Render the current view
        terminal.draw(|f| view(&mut model, f))?;

        // Handle events and map to a Message
        let mut current_msg = handle_event(&model)?;

        // Process updates as long as they return a non-None message
        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }
    stdout().execute(DisableMouseCapture)?;
    tui::restore_terminal()?;
    Ok(())
}

fn fetch_endpoints_from_spec(spec: &Mapping) -> Vec<Endpoint> {
    let mut table_items: Vec<Endpoint> = Vec::new();
    let paths = spec
        .get(&Value::String("paths".to_string()))
        .and_then(|v| v.as_mapping())
        .ok_or_eyre("No 'paths' field found or it's not a mapping")
        .unwrap();

    for (path, ops) in paths {
        let path_str = path
            .as_str()
            .ok_or_eyre("Path key is not a string")
            .unwrap();
        let mut table_item = Endpoint::default();
        let ops_map = ops
            .as_mapping()
            .ok_or_eyre(format!("Operations for '{}' not a mapping", path_str))
            .unwrap();
        let mut refs: Vec<String> = Vec::new();
        for (ops_method, op) in ops_map {
            let mut method = Method::default();
            let method_str = ops_method
                .as_str()
                .ok_or_eyre("Method key is not a string")
                .unwrap();
            if method_str == "parameters" {
                continue;
            }
            if method_str == "summary" {
                table_item.description = op.as_str().unwrap_or("").to_string();
                continue;
            }
            if method_str == "description" && table_item.description.len() == 0 {
                table_item.description = op.as_str().unwrap_or("").to_string();
                continue;
            }
            let summary = op
                .as_mapping()
                .and_then(|m| m.get(&Value::String("summary".to_string())))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let description = op
                .as_mapping()
                .and_then(|m| m.get(&Value::String("description".to_string())))
                .and_then(|v| v.as_str())
                .unwrap_or("No description")
                .to_string();
            method.method = method_str.to_string();
            method.description = if summary.len() > 0 {
                summary
            } else {
                description
            };
            refs.extend(fetch_all_references(op));
            table_item.methods.push(method);
        }
        table_item.path = path_str.to_string();
        table_item.refs = strip_path_from_references(&refs)
            .into_iter()
            .unique()
            .collect();
        table_items.push(table_item);
    }

    // Order table items by path
    table_items.sort_by_key(|item| item.path.clone());
    table_items
}

/// Recursively fetch all $ref values from a Value tree
fn fetch_all_references(value: &Value) -> Vec<String> {
    let mut refs = Vec::new();
    match value {
        Value::Mapping(map) => {
            // Check if this mapping has a $ref key
            if let Some(Value::String(ref_str)) = map.get(&Value::String("$ref".to_string())) {
                refs.push(ref_str.clone());
            }
            // Recurse into all values in the mapping
            for (_, v) in map {
                refs.extend(fetch_all_references(v));
            }
        }
        Value::Sequence(seq) => {
            // Recurse into sequence items
            for item in seq {
                refs.extend(fetch_all_references(item));
            }
        }
        _ => {} // Scalars (String, Number, Bool, Null) have no refs
    }
    refs
}

fn view(model: &mut AppModel, frame: &mut Frame) {
    let [top, bottom] = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
        .areas(frame.area());
    render_table(model, top, frame);
    render_detail(model, bottom, frame);
}

fn render_table(model: &mut AppModel, area: Rect, frame: &mut Frame) {
    let header = Row::new(vec!["    Summary", "Path", "Methods"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let rows = model.table_items.iter().map(|data| {
        let mut description = data.description.clone();
        if description.len() == 0 {
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
            Style::default().fg(Color::Green)
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
            .title(format!(" Endpoints for {}", model.infile))
            .title_alignment(Alignment::Center),
    );

    // Store the table area for pagination
    model.table_area = Some(area);

    frame.render_stateful_widget(table, area, &mut model.table_state);
}

fn render_detail(model: &AppModel, area: Rect, frame: &mut Frame) {
    let selected_item = &model.table_items[model.table_state.selected().unwrap()];
    let mut description = selected_item.description.clone();
    if description.len() == 0 {
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
    detail_lines.push(Line::from("".to_string()));

    let mut refs_lines: Vec<Line> = Vec::new();
    for reference in selected_item.refs.iter() {
        refs_lines.push(Line::from(format!("- {}", reference)));
    }
    if refs_lines.len() > 0 {
        detail_lines.push(Line::from("".to_string()));
        detail_lines.push(Line::from("Component schemas:".to_string()));
        detail_lines.extend(refs_lines);
    }

    let collapsed_top_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        ..symbols::border::PLAIN
    };

    let shortcuts = shortcuts::Shortcuts::from(vec![
        ("q", "quit"),
        ("space", "✂️ snip"),
        ("w", "write and quit"),
        ("↑", "move up"),
        ("↓", "move down"),
    ]);

    let detail = Paragraph::new(Text::from(detail_lines)).block(
        Block::default()
            .border_set(collapsed_top_border_set)
            .borders(Borders::ALL)
            .title_bottom(shortcuts.as_line())
            .title_alignment(Alignment::Right)
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(detail, area);
}

fn handle_event(_: &AppModel) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        match event::read()? {
            Event::Key(key) if key.kind == event::KeyEventKind::Press => Ok(handle_key(key)),
            Event::Mouse(mouse) => Ok(handle_mouse(mouse)),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

const fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNext),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevious),
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('w') => Some(Message::WriteAndQuit),
        KeyCode::Char(' ') => Some(Message::ToggleSelectItemAndSelectNext),
        KeyCode::PageDown => Some(Message::SelectNextPage),
        KeyCode::PageUp => Some(Message::SelectPreviousPage),
        _ => None,
    }
}

const fn handle_mouse(mouse: event::MouseEvent) -> Option<Message> {
    match mouse.kind {
        MouseEventKind::ScrollDown => Some(Message::SelectPrevious),
        MouseEventKind::ScrollUp => Some(Message::SelectNext),
        MouseEventKind::Down(_) => Some(Message::SelectRow(mouse.row)),
        _ => None,
    }
}

fn update(model: &mut AppModel, msg: Message) -> Option<Message> {
    match msg {
        Message::WriteAndQuit => {
            write_spec_to_file(&model).unwrap_or_else(|e| {
                eprintln!("Failed to write spec to file: {}", e);
                model.running_state = RunningState::Done;
            });
            model.running_state = RunningState::Done;
        }
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
        Message::SelectNext => {
            let current_index = model.table_state.selected().unwrap_or(0);
            if current_index < model.table_items.len() - 1 {
                model.table_state.select(Some(current_index + 1));
            }
        }
        Message::SelectPrevious => {
            let current_index = model.table_state.selected().unwrap_or(0);
            if current_index > 0 {
                model.table_state.select(Some(current_index - 1));
            }
        }
        Message::SelectRow(row) => {
            // Subtract 2 because first row is border and the second row is the header
            let row_index = row as usize - 2;
            let scroll_offset = model.table_state.offset();
            let actual_index = row_index + scroll_offset;
            if actual_index < model.table_items.len() {
                model.table_state.select(Some(actual_index));
            }
        }
        Message::ToggleSelectItemAndSelectNext => {
            let current_index = model.table_state.selected().unwrap_or(0);
            model.table_items[current_index].status =
                if model.table_items[current_index].status == Status::Selected {
                    Status::Unselected
                } else {
                    Status::Selected
                };
            if current_index < model.table_items.len() - 1 {
                model.table_state.select(Some(current_index + 1));
            }
        }
        Message::SelectNextPage => {
            let visible_rows = calculate_visible_table_rows(model);
            model.table_state.scroll_down_by(visible_rows);
        }
        Message::SelectPreviousPage => {
            let visible_rows = calculate_visible_table_rows(model);
            model.table_state.scroll_up_by(visible_rows);
        }
    };
    None
}

fn strip_path_from_references(references: &[String]) -> Vec<String> {
    references
        .iter()
        .map(|ref_str| ref_str.split('/').last().unwrap().to_string())
        .collect::<Vec<String>>()
}

fn write_spec_to_file(model: &AppModel) -> color_eyre::Result<()> {
    // Get selected items
    let selected_items: Vec<&Endpoint> = model
        .table_items
        .iter()
        .filter(|item| item.status == Status::Selected)
        .collect();

    // Get the original paths and operations from spec
    let original_path_specifications = model
        .spec
        .get(&Value::String("paths".to_string()))
        .and_then(|v| v.as_mapping())
        .unwrap();

    // Create paths mapping with only selected paths, keeping their original data
    let mut paths = Mapping::new();
    for item in selected_items {
        if let Some(path_data) = original_path_specifications.get(&Value::String(item.path.clone()))
        {
            paths.insert(Value::String(item.path.clone()), path_data.clone());
        }
    }

    // Collect all references from the selected items
    let collected_references_as_schema_index = model
        .table_items
        .iter()
        .filter(|item| item.status == Status::Selected)
        .map(|item| item.refs.clone())
        .flatten()
        .unique()
        .collect::<Vec<String>>();

    // Under components/schemas, there are definitions for all the references
    // But these definitions can also contain references, so we need to collect all of them
    let mut all_references_to_preserve = Vec::new();
    let components = model
        .spec
        .get(&Value::String("components".to_string()))
        .and_then(|v| v.as_mapping())
        .unwrap();
    for (key, value) in components {
        if key.as_str() == Some("schemas") {
            if let Some(schema) = value.as_mapping() {
                for (schema_key, schema_value) in schema {
                    if collected_references_as_schema_index
                        .contains(&schema_key.as_str().unwrap().to_string())
                    {
                        all_references_to_preserve.extend(fetch_all_references(schema_value));
                    }
                }
            }
        }
    }

    // Extract schema names from all references
    let mut all_references_as_schema_index =
        strip_path_from_references(&all_references_to_preserve);
    all_references_as_schema_index.extend(collected_references_as_schema_index);

    // Remove duplicates
    all_references_as_schema_index.sort();
    all_references_as_schema_index.dedup();

    let mut output = Mapping::new();
    // Copy all elements from the spec except paths
    for (key, value) in &model.spec {
        if key.as_str() != Some("paths") {
            if key.as_str() == Some("components") {
                let mut components_output = Mapping::new();
                // Iterate over all children of components and insert them into the output
                for (child_key, child_value) in value.as_mapping().unwrap() {
                    // Unless the child key is schemas
                    if child_key.as_str() != Some("schemas") {
                        components_output.insert(child_key.clone(), child_value.clone());
                    } else {
                        // Iterate over all children of schemas and insert them into the output
                        // But only if the schema key is in all_references_as_schema_index
                        let mut schema_output = Mapping::new();
                        for (schema_key, schema_value) in child_value.as_mapping().unwrap() {
                            if all_references_as_schema_index
                                .contains(&schema_key.as_str().unwrap().to_string())
                            {
                                schema_output.insert(schema_key.clone(), schema_value.clone());
                            }
                        }
                        components_output.insert(child_key.clone(), Value::Mapping(schema_output));
                    }
                }
                output.insert(key.clone(), Value::Mapping(components_output));
            } else {
                output.insert(key.clone(), value.clone());
            }
        } else {
            output.insert(
                Value::String("paths".to_string()),
                Value::Mapping(paths.clone()),
            );
        }
    }

    // Write to file
    use eyre::WrapErr;
    let output_yaml = serde_yaml::to_string(&output).wrap_err("Failed to serialize output");
    fs::write(&model.outfile, output_yaml.unwrap())
        .wrap_err("Failed to write output to file")
        .unwrap();
    Ok(())
}

fn calculate_visible_table_rows(model: &AppModel) -> u16 {
    // Each row is 1 line high, header is 1 line, borders are 2 lines
    let total_rows = model.table_items.len() as u16;
    let visible_rows = model
        .table_area
        .map(|area| area.height.saturating_sub(3))
        .unwrap_or(1);
    visible_rows.min(total_rows)
}

fn styled_method(method: &Method) -> Line {
    let method_str = method.method.to_uppercase();
    let padded_method = format!("{:<6}", method_str);
    let the_method = Span::from(padded_method);
    match method_str.as_str() {
        "GET" => Line::from(vec![
            the_method.style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(" "),
            Span::from(method.description.clone()),
        ]),
        "PATCH" => Line::from(vec![
            the_method.style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(" "),
            Span::from(method.description.clone()),
        ]),
        "POST" => Line::from(vec![
            the_method.style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(" "),
            Span::from(method.description.clone()),
        ]),
        "PUT" => Line::from(vec![
            the_method.style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(" "),
            Span::from(method.description.clone()),
        ]),
        "DELETE" => Line::from(vec![
            the_method.style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::from(" "),
            Span::from(method.description.clone()),
        ]),
        _ => Line::from(vec![
            the_method.style(Style::default().add_modifier(Modifier::ITALIC | Modifier::BOLD)),
            Span::from(" "),
            Span::from(method.description.clone()),
        ]),
    }
}

mod tui {
    use std::io::stdout;
    use std::panic;

    use ratatui::Terminal;
    use ratatui::backend::{Backend, CrosstermBackend};
    use ratatui::crossterm::ExecutableCommand;
    use ratatui::crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
