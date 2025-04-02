use std::io::stdout;

mod event;
mod file;
mod spec_processor;
mod ui;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::ExecutableCommand;
use event::{handle_event, Message};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::TableState;
use ratatui::Frame;
use serde_yaml::Mapping;
use spec_processor::{Endpoint, Status};
use tui_textarea::TextArea;
use crate::ui::{render_detail, render_search, render_table};

#[derive(Default, Clone)]
pub struct SearchState {
    pub(crate) active: bool,
    pub(crate) text_input: TextArea<'static>,
}

#[derive(Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

struct AppModel {
    infile: String,
    outfile: String,
    running_state: RunningState,
    spec: Mapping,
    table_area: Option<ratatui::layout::Rect>,
    table_items: Vec<Endpoint>,
    table_items_backup: Option<Vec<Endpoint>>,
    table_state: TableState,
    search_state: SearchState,
    matcher: SkimMatcherV2,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            infile: String::new(),
            outfile: String::new(),
            running_state: RunningState::default(),
            spec: Mapping::new(),
            table_area: None,
            table_items: Vec::new(),
            table_items_backup: None,
            table_state: TableState::default(),
            search_state: SearchState::default(),
            matcher: SkimMatcherV2::default(),
        }
    }
}

/// Trim an API surface down to size
#[derive(Parser)]
#[clap(version, about = about_str())]
pub struct Args {
    /// The name of the input file or URL
    #[clap()]
    input: String,

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

    let spec = file::read_spec(&args.input)?;

    let mut model = AppModel {
        infile: args.input,
        outfile: args.outfile,
        spec,
        ..Default::default()
    };
    model.table_items = spec_processor::fetch_endpoints_from_spec(&model.spec);
    // Don't preemptively create backup, only when search starts

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
            Constraint::Min(10),
        ])
            .areas(frame.area());
        render_table(model, top, frame);
        render_search(model, search, frame);
        render_detail(model, bottom, frame);
    } else {
        let [top, bottom] = Layout::vertical([
            Constraint::Percentage(80),
            Constraint::Min(10),
        ])            .areas(frame.area());
        render_table(model, top, frame);
        render_detail(model, bottom, frame);
    }
}

impl AppModel {
    // Helper method to maintain selection when items are reordered
    fn maintain_selection(&mut self, path_to_follow: &str) {
        if let Some(new_idx) = self.table_items.iter().position(|item| item.path == path_to_follow) {
            self.table_state.select(Some(new_idx));
        }
    }
    
    // Helper method to ensure selection is valid
    fn ensure_valid_selection(&mut self) {
        if self.table_items.is_empty() {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected() {
            if selected >= self.table_items.len() {
                self.table_state.select(Some(0));
            }
        } else if !self.table_items.is_empty() {
            self.table_state.select(Some(0));
        }
    }
    
    // Helper to update item status in both table_items and backup
    fn toggle_item_status(&mut self, index: usize) -> (String, Status) {
        let path = self.table_items[index].path.clone();
        let new_status = if self.table_items[index].status == Status::Selected {
            Status::Unselected
        } else {
            Status::Selected
        };
        
        // Update in current display
        self.table_items[index].status = new_status;
        
        // Update in backup if it exists
        if let Some(backup) = &mut self.table_items_backup {
            if let Some(pos) = backup.iter().position(|item| item.path == path) {
                backup[pos].status = new_status;
            }
        }
        
        (path, new_status)
    }
    
    // Filter items based on query and maintain selection
    fn filter_items(&mut self, query: &str) {
        // Remember current selection
        let selected_path = self.table_state.selected()
            .and_then(|idx| self.table_items.get(idx))
            .map(|item| item.path.clone());
        
        // Ensure backup exists
        if self.table_items_backup.is_none() {
            self.table_items_backup = Some(self.table_items.clone());
        }
        
        let backup = self.table_items_backup.as_ref().unwrap();
        
        if query.is_empty() {
            // Reset to full list when query is empty
            self.table_items = backup.clone();
            sort_items_selected_first(&mut self.table_items);
        } else {
            // Filter with weighted scoring
            let mut scored_items = backup
                .iter()
                .filter_map(|item| {
                    let path_score = self.matcher.fuzzy_match(&item.path.to_lowercase(), query);
                    let desc_score = self.matcher.fuzzy_match(&item.description.to_lowercase(), query);
                    
                    match (path_score, desc_score) {
                        (Some(p), Some(d)) => Some((item, p * 2 + d)),  // Path counts double
                        (Some(p), None)    => Some((item, p * 2)),
                        (None, Some(d))    => Some((item, d)),
                        (None, None)       => None,
                    }
                })
                .collect::<Vec<_>>();
            
            // Sort: selected first, then by score
            scored_items.sort_by(|a, b| {
                match (a.0.status, b.0.status) {
                    (Status::Selected, Status::Unselected) => std::cmp::Ordering::Less,
                    (Status::Unselected, Status::Selected) => std::cmp::Ordering::Greater,
                    _ => b.1.cmp(&a.1), // Higher score first
                }
            });
            
            // Extract just items
            self.table_items = scored_items
                .into_iter()
                .map(|(item, _)| item.clone())
                .collect();
        }
        
        // Try to maintain selection
        if let Some(path) = selected_path {
            self.maintain_selection(&path);
        }
        
        self.ensure_valid_selection();
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
            None
        }
        
        Message::Quit => {
            model.running_state = RunningState::Done;
            None
        }
        
        Message::GoToTop => {
            if model.table_items.is_empty() {
                return None;
            }
            
            // Reset to first item and scroll to the top
            model.table_state.select(Some(0));
            
            // Scroll all the way to the top
            let current_offset = model.table_state.offset();
            if current_offset > 0 {
                model.table_state.scroll_up_by(current_offset as u16);
            }
            None
        }
        
        Message::SelectNext => {
            if model.table_items.is_empty() {
                return None;
            }
            
            let current_index = model.table_state.selected().unwrap_or(0);
            if current_index < model.table_items.len() - 1 {
                model.table_state.select(Some(current_index + 1));
            }
            None
        }
        
        Message::SelectPrevious => {
            if model.table_items.is_empty() {
                return None;
            }
            
            let current_index = model.table_state.selected().unwrap_or(0);
            if current_index > 0 {
                model.table_state.select(Some(current_index - 1));
            }
            None
        }
        
        Message::SelectRow(row) => {
            // Skip if clicked outside the table content area
            let row_offset = 2; // First row is border, second is header
            let last_index = model.table_area
                .map(|area| area.height.saturating_sub(1))
                .unwrap_or(1);
                
            if row < row_offset || row > last_index {
                return None;
            }
            
            let row_index = row - row_offset;
            let scroll_offset = model.table_state.offset();
            let actual_index = (row_index + scroll_offset as u16) as usize;
            
            if actual_index < model.table_items.len() {
                model.table_state.select(Some(actual_index));
            }
            None
        }
        
        Message::ToggleSelectItemAndSelectNext => {
            // Skip if no selection or empty list
            if model.table_items.is_empty() || model.table_state.selected().is_none() {
                return None;
            }
            
            let current_index = model.table_state.selected().unwrap();
            if current_index >= model.table_items.len() {
                return None;
            }
            
            // Remember next item's path before changes
            let next_item_path = if current_index < model.table_items.len() - 1 {
                Some(model.table_items[current_index + 1].path.clone())
            } else {
                None
            };
            
            // Toggle status of current item
            let (current_path, _) = model.toggle_item_status(current_index);
            
            // Move to next item before reordering
            if current_index < model.table_items.len() - 1 {
                model.table_state.select(Some(current_index + 1));
            }
            
            // Reorder items if not in search mode
            if !model.search_state.active {
                // Use next item for focus if available, otherwise use current
                let focused_path = next_item_path.unwrap_or(current_path);
                
                // Sort selected items to top
                sort_items_selected_first(&mut model.table_items);
                
                // Maintain selection on the focused item
                model.maintain_selection(&focused_path);
            }
            None
        }
        
        Message::SelectNextPage => {
            if !model.table_items.is_empty() {
                let visible_rows = calculate_visible_table_rows(model);
                model.table_state.scroll_down_by(visible_rows);
            }
            None
        }
        
        Message::SelectPreviousPage => {
            if !model.table_items.is_empty() {
                let visible_rows = calculate_visible_table_rows(model);
                model.table_state.scroll_up_by(visible_rows);
            }
            None
        }
        
        Message::ShowSearch => {
            model.search_state.active = true;
            model.search_state.text_input = TextArea::default();
            
            // Backup the current table items if not already backed up
            if model.table_items_backup.is_none() {
                model.table_items_backup = Some(model.table_items.clone());
            }
            None
        }
        
        Message::HideSearch => {
            // Remember current selection
            let selected_path = model.table_state.selected()
                .and_then(|idx| model.table_items.get(idx))
                .map(|item| item.path.clone());
            
            model.search_state.active = false;
            model.search_state.text_input = TextArea::default();
            
            // Restore items and sort selected to top
            if let Some(backup) = &model.table_items_backup {
                model.table_items = backup.clone();
                sort_items_selected_first(&mut model.table_items);
            }
            
            // Try to maintain selection
            if let Some(path) = selected_path {
                model.maintain_selection(&path);
            }
            
            model.ensure_valid_selection();
            None
        }
        
        Message::KeyPress(key) => {
            model.search_state.text_input.input(key);
            
            if model.search_state.active {
                let query = model.search_state.text_input.lines()
                    .get(0)
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();
                
                model.filter_items(&query);
            }
            None
        }
    }
}

// Helper function to sort items with selected ones at the top
fn sort_items_selected_first(items: &mut Vec<Endpoint>) {
    items.sort_by(|a, b| match (a.status, b.status) {
        (Status::Selected, Status::Unselected) => std::cmp::Ordering::Less,
        (Status::Unselected, Status::Selected) => std::cmp::Ordering::Greater,
        _ => a.path.cmp(&b.path), // Sort by path when selection status is the same
    });
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
