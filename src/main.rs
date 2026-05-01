mod app;
mod error;
mod event;
mod reader;
mod table;
mod ui;
mod writer;

use std::{env, io, path::Path};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use app::{App, CommandResult, Mode};
use event::{poll_event, AppEvent};
use error::AppError;

fn main() -> Result<(), AppError> {
    let path_str = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Uso: excel-tui <arquivo.csv|.xlsx|.xls>");
        std::process::exit(1);
    });

    let path   = Path::new(&path_str);
    let reader = reader::reader_for(path)?;
    let data   = reader.read(path)?;
    let mut app = App::new(data);

    run_tui(&mut app, path)?;
    Ok(())
}

fn run_tui(app: &mut App, path: &Path) -> Result<(), AppError> {
    enable_raw_mode().map_err(AppError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(AppError::Io)?;

    let backend      = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(AppError::Io)?;

    let result = event_loop(&mut terminal, app, path);

    disable_raw_mode().map_err(AppError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(AppError::Io)?;
    terminal.show_cursor().map_err(AppError::Io)?;

    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    path: &Path,
) -> Result<(), AppError> {
    loop {
        terminal.draw(|f| ui::draw(f, app)).map_err(AppError::Io)?;

        if let Some(ev) = poll_event(50)? {
            match app.mode.clone() {
                Mode::Insert  => handle_insert(app, ev),
                Mode::Command => handle_command(app, ev, path)?,
                Mode::Search  => handle_search(app, ev),
                _             => handle_normal(app, ev, path)?,
            }
        }

        if app.should_quit { break; }
    }
    Ok(())
}

// ── Normal mode ───────────────────────────────────────────────────────────────

fn handle_normal(app: &mut App, ev: AppEvent, path: &Path) -> Result<(), AppError> {
    match ev {
        AppEvent::Quit => {
            if app.dirty {
                app.status_msg = Some("Arquivo modificado. Use :w para salvar ou :q! para sair.".into());
            } else {
                app.should_quit = true;
            }
        }
        AppEvent::ScrollDown  => app.scroll_down(),
        AppEvent::ScrollUp    => app.scroll_up(),
        AppEvent::ScrollLeft  => app.scroll_left(),
        AppEvent::ScrollRight => app.scroll_right(),
        AppEvent::GotoLastRow  => app.goto_last_row(),
        AppEvent::GotoFirstCol => app.goto_first_col(),
        AppEvent::GotoLastCol  => app.goto_last_col(),
        AppEvent::NextSheet    => app.next_sheet(),
        AppEvent::PrevSheet    => app.prev_sheet(),
        AppEvent::EnterSearch  => app.enter_search(),
        AppEvent::SearchNext   => app.search_next(),
        AppEvent::SearchPrev   => app.search_prev(),
        AppEvent::ToggleHelp   => app.toggle_help(),
        AppEvent::CtrlS        => save_file(app, path)?,
        AppEvent::CtrlR        => app.redo(),
        AppEvent::Char(c) => {
            match c {
                // sequências vim: gg / dd / yy
                'g' | 'd' | 'y' => { app.handle_pending_key(c); }
                // edição
                'i' => app.enter_insert(),
                'o' => app.insert_row_below(),
                'O' => app.insert_row_above(),
                'x' => app.clear_cell(),
                'p' => app.paste_below(),
                'P' => app.paste_above(),
                'u' => app.undo(),
                ':' => app.enter_command(),
                _   => { app.pending_key = None; }
            }
        }
        AppEvent::ExitMode => { app.pending_key = None; app.status_msg = None; }
        _ => {}
    }
    Ok(())
}

// ── Insert mode ───────────────────────────────────────────────────────────────

fn handle_insert(app: &mut App, ev: AppEvent) {
    match ev {
        AppEvent::Enter     => app.confirm_edit(),
        AppEvent::ExitMode  => app.cancel_edit(),
        AppEvent::Backspace => app.edit_backspace(),
        AppEvent::Char(c)   => app.edit_push(c),
        _ => {}
    }
}

// ── Command mode ──────────────────────────────────────────────────────────────

fn handle_command(app: &mut App, ev: AppEvent, path: &Path) -> Result<(), AppError> {
    match ev {
        AppEvent::Enter => {
            match app.execute_command() {
                CommandResult::Save => {
                    save_file(app, path)?;
                }
                CommandResult::Quit => {
                    if app.dirty {
                        app.status_msg = Some("Modificações não salvas. Use :w ou :q!".into());
                    } else {
                        app.should_quit = true;
                    }
                }
                CommandResult::SaveQuit => {
                    save_file(app, path)?;
                    app.should_quit = true;
                }
                CommandResult::ForceQuit => {
                    app.should_quit = true;
                }
                CommandResult::Unknown(cmd) => {
                    app.status_msg = Some(format!("Comando desconhecido: {}", cmd));
                }
                CommandResult::None => {}
            }
        }
        AppEvent::ExitMode  => app.cancel_command(),
        AppEvent::Backspace => app.command_backspace(),
        AppEvent::Char(c)   => app.command_push(c),
        _ => {}
    }
    Ok(())
}

// ── Search mode ───────────────────────────────────────────────────────────────

fn handle_search(app: &mut App, ev: AppEvent) {
    match ev {
        AppEvent::ExitMode | AppEvent::Enter => app.exit_search(),
        AppEvent::Char(c)   => {
            let mut q = app.search_query.clone();
            q.push(c);
            app.update_search(&q);
        }
        AppEvent::Backspace => {
            let mut q = app.search_query.clone();
            q.pop();
            app.update_search(&q);
        }
        AppEvent::SearchNext => app.search_next(),
        AppEvent::SearchPrev => app.search_prev(),
        _ => {}
    }
}

// ── Save ──────────────────────────────────────────────────────────────────────

fn save_file(app: &mut App, path: &Path) -> Result<(), AppError> {
    let w = writer::writer_for(path)?;
    w.write(&app.data, path)?;
    app.dirty = false;
    app.status_msg = Some(format!("Salvo: {}", path.display()));
    Ok(())
}
