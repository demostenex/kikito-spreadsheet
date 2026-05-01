use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    Quit,
    ScrollDown,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
    GotoFirstRow,
    GotoLastRow,
    GotoFirstCol,
    GotoLastCol,
    NextSheet,
    PrevSheet,
    EnterSearch,
    SearchNext,
    SearchPrev,
    ExitSearch,
    Char(char),
    Backspace,
    ToggleHelp,
    Resize,
}

pub fn poll_event(timeout_ms: u64) -> Result<Option<AppEvent>, AppError> {
    if !event::poll(Duration::from_millis(timeout_ms)).map_err(AppError::Io)? {
        return Ok(None);
    }
    match event::read().map_err(AppError::Io)? {
        Event::Key(KeyEvent { code, modifiers, .. }) => Ok(Some(map_key(code, modifiers))),
        Event::Resize(_, _) => Ok(Some(AppEvent::Resize)),
        _ => Ok(None),
    }
}

fn map_key(code: KeyCode, modifiers: KeyModifiers) -> AppEvent {
    match code {
        KeyCode::Char('q') | KeyCode::Char('c')
            if modifiers.contains(KeyModifiers::CONTROL) => AppEvent::Quit,
        KeyCode::Char('q')                => AppEvent::Quit,
        KeyCode::Char('j') | KeyCode::Down  => AppEvent::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up    => AppEvent::ScrollUp,
        KeyCode::Char('h') | KeyCode::Left  => AppEvent::ScrollLeft,
        KeyCode::Char('l') | KeyCode::Right => AppEvent::ScrollRight,
        KeyCode::Char('g')                => AppEvent::GotoFirstRow,
        KeyCode::Char('G')                => AppEvent::GotoLastRow,
        KeyCode::Char('0')                => AppEvent::GotoFirstCol,
        KeyCode::Char('$')                => AppEvent::GotoLastCol,
        KeyCode::Tab                      => AppEvent::NextSheet,
        KeyCode::BackTab                  => AppEvent::PrevSheet,
        KeyCode::Char('/')                => AppEvent::EnterSearch,
        KeyCode::Char('n')                => AppEvent::SearchNext,
        KeyCode::Char('N')                => AppEvent::SearchPrev,
        KeyCode::Esc                      => AppEvent::ExitSearch,
        KeyCode::Char('?')                => AppEvent::ToggleHelp,
        KeyCode::Backspace                => AppEvent::Backspace,
        KeyCode::Char(c)                  => AppEvent::Char(c),
        _                                 => AppEvent::Char('\0'),
    }
}
