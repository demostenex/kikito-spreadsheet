mod app;
mod error;
mod event;
mod reader;
mod table;
mod ui;

use std::{env, io, path::Path};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use app::{App, Mode};
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

    run_tui(&mut app)?;
    Ok(())
}

fn run_tui(app: &mut App) -> Result<(), AppError> {
    enable_raw_mode().map_err(AppError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(AppError::Io)?;

    let backend  = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(AppError::Io)?;

    let result = event_loop(&mut terminal, app);

    disable_raw_mode().map_err(AppError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(AppError::Io)?;
    terminal.show_cursor().map_err(AppError::Io)?;

    result
}

fn event_loop(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), AppError> {
    loop {
        terminal.draw(|f| ui::draw(f, app)).map_err(AppError::Io)?;

        if let Some(ev) = poll_event(50)? {
            match app.mode {
                Mode::Search => handle_search(app, ev),
                _            => handle_normal(app, ev),
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_normal(app: &mut App, ev: AppEvent) {
    match ev {
        AppEvent::Quit        => app.should_quit = true,
        AppEvent::ScrollDown  => app.scroll_down(),
        AppEvent::ScrollUp    => app.scroll_up(),
        AppEvent::ScrollLeft  => app.scroll_left(),
        AppEvent::ScrollRight => app.scroll_right(),
        AppEvent::GotoFirstRow => app.goto_first_row(),
        AppEvent::GotoLastRow  => app.goto_last_row(),
        AppEvent::GotoFirstCol => app.goto_first_col(),
        AppEvent::GotoLastCol  => app.goto_last_col(),
        AppEvent::NextSheet   => app.next_sheet(),
        AppEvent::PrevSheet   => app.prev_sheet(),
        AppEvent::EnterSearch => app.enter_search(),
        AppEvent::SearchNext  => app.search_next(),
        AppEvent::SearchPrev  => app.search_prev(),
        AppEvent::ToggleHelp  => app.toggle_help(),
        _                     => {}
    }
}

fn handle_search(app: &mut App, ev: AppEvent) {
    match ev {
        AppEvent::ExitSearch | AppEvent::Quit => app.exit_search(),
        AppEvent::Char(c) => {
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
