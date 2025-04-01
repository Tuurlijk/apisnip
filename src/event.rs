use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use color_eyre::Result;

#[derive(PartialEq, Copy, Clone)]
pub enum Message {
    SelectNext,
    SelectPrevious,
    SelectRow(u16),
    ToggleSelectItemAndSelectNext,
    SelectNextPage,
    SelectPreviousPage,
    WriteAndQuit,
    Quit,
}

pub fn handle_event() -> Result<Option<Message>> {
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