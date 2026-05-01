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
    GotoFirstCol,
    GotoLastCol,
    GotoLastRow,
    NextSheet,
    PrevSheet,
    EnterSearch,
    SearchNext,
    SearchPrev,
    ExitMode,       // Esc — sai de Insert / Search / Command / Help
    Enter,
    Char(char),
    Backspace,
    ToggleHelp,
    CtrlS,          // salvar atalho
    CtrlR,          // redo
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
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);

    match code {
        KeyCode::Char('c') if ctrl => AppEvent::Quit,
        KeyCode::Char('s') if ctrl => AppEvent::CtrlS,
        KeyCode::Char('r') if ctrl => AppEvent::CtrlR,

        KeyCode::Char('q')                 => AppEvent::Quit,
        KeyCode::Char('j') | KeyCode::Down  => AppEvent::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up    => AppEvent::ScrollUp,
        KeyCode::Char('h') | KeyCode::Left  => AppEvent::ScrollLeft,
        KeyCode::Char('l') | KeyCode::Right => AppEvent::ScrollRight,
        KeyCode::Char('0')                 => AppEvent::GotoFirstCol,
        KeyCode::Char('$')                 => AppEvent::GotoLastCol,
        KeyCode::Char('G')                 => AppEvent::GotoLastRow,
        KeyCode::Tab                       => AppEvent::NextSheet,
        KeyCode::BackTab                   => AppEvent::PrevSheet,
        KeyCode::Char('/')                 => AppEvent::EnterSearch,
        KeyCode::Char('n')                 => AppEvent::SearchNext,
        KeyCode::Char('N')                 => AppEvent::SearchPrev,
        KeyCode::Esc                       => AppEvent::ExitMode,
        KeyCode::Enter                     => AppEvent::Enter,
        KeyCode::Char('?')                 => AppEvent::ToggleHelp,
        KeyCode::Backspace                 => AppEvent::Backspace,
        KeyCode::Char(c)                   => AppEvent::Char(c),
        _                                  => AppEvent::Char('\0'),
    }
}
