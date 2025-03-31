use std::fs;
use std::io::stdout;
/// A Ratatui example that demonstrates the Elm architecture with a basic list - detail
/// application.
///
/// This example runs with the Ratatui library code in the branch that you are currently
/// reading. See the [`latest`] branch for the code which works with the most recent Ratatui
/// release.
///
/// [`latest`]: https://github.com/ratatui/ratatui/tree/latest
use std::time::Duration;

mod config;

use clap::Parser;
use color_eyre::eyre::OptionExt;
use crossterm::ExecutableCommand;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
use itertools::Itertools;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Row, Table, TableState};
use serde_yaml::{self, Mapping, Value};

static INFO_TEXT: &str = " (Esc/q) quit | (↑) move up | (↓) move down ";

/// Application data model and state
#[derive(Debug, Default)]
struct Model {
    table_items: Vec<Data>,
    table_state: TableState,
    running_state: RunningState,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

/// Data model
#[derive(Debug, Default)]
struct Data {
    methods: Vec<Method>,
    path: String,
    description: String,
    refs: Vec<String>,
}

#[derive(Debug, Default)]
struct Method {
    method: String,
    description: String,
    refs: Vec<String>,
}

#[derive(PartialEq, Copy, Clone)]
enum Message {
    SelectNext,
    SelectPrevious,
    SelectRow(u16),
    Quit,
}

/// Trim an API surface down to size
#[derive(Parser)]
#[clap(version, about = about_str())]
pub struct Args {
    /// The name of the input file
    #[clap(short, long)]
    infile: String,

    /// The name of the output file
    #[clap(short, long)]
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
        r"              
                     _____     _ _____     _     
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

    let mut table_items: Vec<Data> = Vec::new();
    let paths = spec
        .get(&Value::String("paths".to_string()))
        .and_then(|v| v.as_mapping())
        .ok_or_eyre("No 'paths' field found or it's not a mapping")?;

    for (path, ops) in paths {
        let path_str = path.as_str().ok_or_eyre("Path key is not a string")?;
        let mut table_item = Data::default();
        let ops_map = ops
            .as_mapping()
            .ok_or_eyre(format!("Operations for '{}' not a mapping", path_str))?;
        let mut refs: Vec<String> = Vec::new();
        for (ops_method, op) in ops_map {
            let mut method = Method::default();
            let method_str = ops_method
                .as_str()
                .ok_or_eyre("Method key is not a string")?;
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

    let mut terminal = tui::init_terminal()?;
    let mut model = Model {
        table_items: table_items,
        ..Default::default()
    };
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

// Fetch the $ref: values from the operation

// Recursively fetch all $ref values from a Value tree
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
    let header = Row::new(vec!["Summary", "Path", "Summary"])
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
        Row::new(vec![
            description,
            data.path.clone(),
            data.methods
                .iter()
                .map(|method| method.method.as_str().to_uppercase())
                .collect::<Vec<String>>()
                .join(" "),
        ])
        .height(1)
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
            .title(" Terminal fans "),
    );

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
        "Method: {}\n\nPath: {}\n\nRefs:\n- {}",
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
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
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
        Message::Quit => model.running_state = RunningState::Done,
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
    };
    None
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
