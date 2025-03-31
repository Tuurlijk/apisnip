use std::fs;
use std::io::stdout;
use std::time::Duration;

mod config;

use clap::Parser;
use color_eyre::eyre::{self, OptionExt};
use crossterm::ExecutableCommand;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
use itertools::Itertools;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Row, Table, TableState};
use serde_yaml::{self, Mapping, Value};

static INFO_TEXT: &str = " (q) quit | (space/Enter) toggle select and move to next | (w) write and quit | (↑) move up | (↓) move down | (PageUp/PageDown) move page up/down ";

/// Application data model and state
#[derive(Debug, Default)]
struct Model {
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

/// Data model
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
    ToggleSelectItem,
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
    let config = config::get_config();
    let args = Args::parse();
    let input_yaml = fs::read_to_string(&args.infile)?;
    let spec: Mapping = serde_yaml::from_str(&input_yaml)?;
    let mut model = Model {
        infile: args.infile,
        outfile: args.outfile,
        spec: spec,
        ..Default::default()
    };
    model.table_items = fetch_endpoints_from_spec(&model.spec);

    let mut terminal = tui::init_terminal()?;
    stdout().execute(EnableMouseCapture)?;
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
            refs.extend(fetch_refs(op));
            table_item.methods.push(method);
        }
        table_item.path = path_str.to_string();
        table_item.refs = refs.into_iter().unique().collect();
        table_items.push(table_item);
    }

    // Order table items by path
    table_items.sort_by_key(|item| item.path.clone());
    table_items
}

/// Fetch the $ref: values from the operation
/// Recursively fetch all $ref values from a Value tree
fn fetch_refs(value: &Value) -> Vec<String> {
    let mut refs = Vec::new();
    match value {
        Value::Mapping(map) => {
            // Check if this mapping has a $ref key
            if let Some(Value::String(ref_str)) = map.get(&Value::String("$ref".to_string())) {
                refs.push(ref_str.clone());
            }
            // Recurse into all values in the mapping
            for (_, v) in map {
                refs.extend(fetch_refs(v));
            }
        }
        Value::Sequence(seq) => {
            // Recurse into sequence items
            for item in seq {
                refs.extend(fetch_refs(item));
            }
        }
        _ => {} // Scalars (String, Number, Bool, Null) have no refs
    }
    refs
}

fn view(model: &mut Model, frame: &mut Frame) {
    let [top, bottom] = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
        .areas(frame.area());

    // Table setup
    let header = Row::new(vec!["Summary", "Path", "Methods"])
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
            Status::Unselected => format!(" ☐ {}", description),
            Status::Selected => format!(" ✓ {}", description),
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
            .borders(Borders::ALL)
            .title(format!(" Endpoints for {}", model.infile))
            .title_alignment(Alignment::Center),
    );

    // Store the table area for pagination
    model.table_area = Some(top);

    frame.render_stateful_widget(table, top, &mut model.table_state);

    // Select the first row if no row is selected
    if model.table_state.selected().is_none() {
        model.table_state.select_first();
    }

    // Detail view
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

    let detail = Paragraph::new(format!(
        "Methods: {}\n\nPath: {}\n\nRefs:\n- {}",
        selected_item
            .methods
            .iter()
            .map(|method| method.method.as_str().to_uppercase())
            .collect::<Vec<String>>()
            .join(" "),
        selected_item.path,
        selected_item.refs.join("\n- ")
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", description))
            .title_bottom(Line::from(INFO_TEXT))
            .padding(Padding::new(1, 1, 1, 1)),
    );
    frame.render_widget(detail, bottom);
}

fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
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
        KeyCode::Char(' ') | KeyCode::Right | KeyCode::Enter => {
            Some(Message::ToggleSelectItemAndSelectNext)
        }
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

fn update(model: &mut Model, msg: Message) -> Option<Message> {
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
        Message::ToggleSelectItem => {
            let current_index = model.table_state.selected().unwrap_or(0);
            model.table_items[current_index].status =
                if model.table_items[current_index].status == Status::Selected {
                    Status::Unselected
                } else {
                    Status::Selected
                };
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

fn write_spec_to_file(model: &Model) -> color_eyre::Result<()> {
    // Get selected items
    let selected_items: Vec<&Endpoint> = model
        .table_items
        .iter()
        .filter(|item| item.status == Status::Selected)
        .collect();

    // Get the original paths from spec
    let original_paths = model
        .spec
        .get(&Value::String("paths".to_string()))
        .and_then(|v| v.as_mapping())
        .unwrap();

    // Create paths mapping with only selected paths, keeping their original data
    let mut paths = Mapping::new();
    for item in selected_items {
        if let Some(path_data) = original_paths.get(&Value::String(item.path.clone())) {
            paths.insert(Value::String(item.path.clone()), path_data.clone());
        }
    }

    let mut output = Mapping::new();
    // Copy all elements from the spec except paths
    for (key, value) in &model.spec {
        if key.as_str() != Some("paths") {
            output.insert(key.clone(), value.clone());
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

fn calculate_visible_table_rows(model: &Model) -> u16 {
    // Each row is 1 line high, header is 1 line, borders are 2 lines
    let total_rows = model.table_items.len() as u16;
    let visible_rows = model
        .table_area
        .map(|area| area.height.saturating_sub(3))
        .unwrap_or(1);
    visible_rows.min(total_rows)
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
