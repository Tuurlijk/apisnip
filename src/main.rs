use std::io::stdout;

mod event;
mod file;
mod spec_processor;
mod ui;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::ExecutableCommand;
use event::{handle_event, Message};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::TableState;
use ratatui::Frame;
use serde_yaml::Mapping;
use spec_processor::{Endpoint, Status};
use tui_textarea::TextArea;
use crate::ui::{render_detail, render_search, render_table};

#[derive(Debug, Default)]
struct AppModel {
    infile: String,
    outfile: String,
    running_state: RunningState,
    spec: Mapping,
    table_area: Option<ratatui::layout::Rect>,
    table_items: Vec<Endpoint>,
    table_state: TableState,
    search_state: SearchState,
}

#[derive(Debug, Default, Clone)]
pub struct SearchState {
    pub(crate) active: bool,
    pub(crate) query: String,
    pub(crate) text_input: TextArea<'static>,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
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

    model.search_state.text_input.insert_str("Cowabunga!");

    let mut terminal = tui::init_terminal()?;
    while model.running_state != RunningState::Done {
        // Render the current view
        terminal.draw(|f| view(&mut model, f))?;

        // Handle events and map to a Message
        let mut current_msg = handle_event(&mut model)?;

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
    if model.search_state.active {
        let [top, search, bottom] = Layout::vertical([
            Constraint::Percentage(80),
            Constraint::Length(2),
            Constraint::Min(9),
        ])
            .areas(frame.area());
        render_table(model, top, frame);
        render_search(model, search, frame);
        render_detail(model, bottom, frame);
    } else {
        let [top, bottom] = Layout::vertical([
            Constraint::Percentage(80),
            Constraint::Min(9),
        ])            .areas(frame.area());
        render_table(model, top, frame);
        render_detail(model, bottom, frame);
    }
}

fn update(model: &mut AppModel, msg: Message) -> Option<Message> {
    match msg {
        Message::WriteAndQuit => {
            file::write_spec_to_file(&model.outfile, &model.spec, &model.table_items)
                .unwrap_or_else(|e| {
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
            let row_offset = 2;
            let last_index = model
                .table_area
                .map(|area| area.height.saturating_sub(1))
                .unwrap_or(1);
            // If the row is less than the offset, or greater than the last index, we don't need to process the message
            if row < row_offset || row > last_index {
                return None;
            }
            let row_index = row - row_offset;
            let scroll_offset: usize = model.table_state.offset();
            let actual_index = row_index + scroll_offset as u16;
            if actual_index < model.table_items.len() as u16 {
                model.table_state.select(Some(actual_index as usize));
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
        Message::ShowSearch => {
            model.search_state.active = true;
        }
        Message::HideSearch => {
            model.search_state.active = false;
        }
        Message::KeyPress(key) => {
            model.search_state.text_input.input(key);
        }
    };
    None
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

    use ratatui::backend::{Backend, CrosstermBackend};
    use ratatui::crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use ratatui::crossterm::ExecutableCommand;
    use ratatui::Terminal;

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
