use std::io::stdout;
use std::time::Duration;

mod file;
mod shortcuts;
mod spec_processor;
mod ui;

use clap::Parser;
use crossterm::ExecutableCommand;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::TableState;
use serde_yaml::Mapping;

use spec_processor::{Endpoint, Status};
use ui::{render_detail, render_table};

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

    let spec = file::read_spec(&args.infile)?;

    let mut model = AppModel {
        infile: args.infile,
        outfile: args.outfile,
        spec,
        ..Default::default()
    };
    model.table_items = spec_processor::fetch_endpoints_from_spec(&model.spec);

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

fn view(model: &mut AppModel, frame: &mut Frame) {
    let [top, bottom] = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
        .areas(frame.area());
    render_table(model, top, frame);
    render_detail(model, bottom, frame);
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
            write_spec_to_file(model).unwrap_or_else(|e| {
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

fn write_spec_to_file(model: &AppModel) -> color_eyre::Result<()> {
    let selected_items: Vec<&Endpoint> = model
        .table_items
        .iter()
        .filter(|item| item.status == Status::Selected)
        .collect();

    let output = spec_processor::process_spec_for_output(&model.spec, &selected_items)?;
    file::write_spec(&model.outfile, &output)
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
