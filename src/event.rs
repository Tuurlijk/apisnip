use crate::AppModel;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEventKind};
use std::time::Duration;

#[derive(PartialEq, Copy, Clone)]
pub enum Message {
    ClearSearch,
    GoToBottom,
    GoToTop,
    HideSearch,
    KeyPress(KeyEvent),
    Quit,
    ScrollDown,
    ScrollUp,
    SelectNext,
    SelectNextPage,
    SelectPrevious,
    SelectPreviousPage,
    SelectRow(u16),
    ShowSearch,
    ToggleSelectItemAndSelectNext,
    WriteAndQuit,
}

pub fn handle_event(model: &mut AppModel) -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        match event::read()? {
            Event::Key(key) if key.kind == event::KeyEventKind::Press => Ok(handle_key(key, model)),
            Event::Mouse(mouse) => Ok(handle_mouse(mouse)),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

const fn handle_key(key: event::KeyEvent, model: &mut AppModel) -> Option<Message> {
    if model.search_state.active {
        match key.code {
            KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                Some(Message::ClearSearch)
            }
            KeyCode::Esc => Some(Message::HideSearch),
            KeyCode::Char(' ') => Some(Message::ToggleSelectItemAndSelectNext),
            KeyCode::Up => Some(Message::SelectPrevious),
            KeyCode::Down => Some(Message::SelectNext),
            KeyCode::PageDown => Some(Message::SelectNextPage),
            KeyCode::PageUp => Some(Message::SelectPreviousPage),
            KeyCode::Home => Some(Message::GoToTop),
            KeyCode::Enter => None,
            _ => Some(Message::KeyPress(key)),
        }
    } else {
        match key.code {
            KeyCode::Char(' ') => Some(Message::ToggleSelectItemAndSelectNext),
            KeyCode::Char('/') => Some(Message::ShowSearch),
            KeyCode::Char('j') => Some(Message::SelectNext),
            KeyCode::Char('k') => Some(Message::SelectPrevious),
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('w') => Some(Message::WriteAndQuit),
            KeyCode::Up => Some(Message::SelectPrevious),
            KeyCode::Down => Some(Message::SelectNext),
            KeyCode::Esc => Some(Message::HideSearch),
            KeyCode::PageDown => Some(Message::SelectNextPage),
            KeyCode::PageUp => Some(Message::SelectPreviousPage),
            KeyCode::Home => Some(Message::GoToTop),
            KeyCode::End => Some(Message::GoToBottom),
            _ => None,
        }
    }
}

const fn handle_mouse(mouse: event::MouseEvent) -> Option<Message> {
    match mouse.kind {
        MouseEventKind::ScrollDown => Some(Message::ScrollDown),
        MouseEventKind::ScrollUp => Some(Message::ScrollUp),
        MouseEventKind::Down(_) => Some(Message::SelectRow(mouse.row)),
        _ => None,
    }
}
