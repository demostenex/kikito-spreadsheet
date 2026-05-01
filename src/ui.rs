use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell as RCell, Clear, Paragraph, Row, Table, TableState},
};
use crate::app::{App, Mode};

const HEADER_BG: Color      = Color::Rgb(30, 30, 60);
const HEADER_FG: Color      = Color::Rgb(180, 180, 255);
const CURSOR_ROW_BG: Color  = Color::Rgb(45, 75, 130);
const ACTIVE_CELL_BG: Color = Color::Rgb(255, 215, 0);
const ACTIVE_CELL_FG: Color = Color::Rgb(10, 10, 10);
const INSERT_CELL_BG: Color = Color::Rgb(0, 180, 100);
const INSERT_CELL_FG: Color = Color::Rgb(0, 0, 0);
const EVEN_BG: Color        = Color::Rgb(18, 18, 28);
const ODD_BG: Color         = Color::Rgb(24, 24, 38);
const STATUS_BG: Color      = Color::Rgb(40, 40, 80);
const SEARCH_BG: Color      = Color::Rgb(60, 40, 0);
const COMMAND_BG: Color     = Color::Rgb(20, 20, 20);
const MATCH_FG: Color       = Color::Rgb(255, 200, 0);
const BORDER_FG: Color      = Color::Rgb(80, 80, 120);
const DIRTY_FG: Color       = Color::Rgb(255, 100, 80);
const INSERT_FG: Color      = Color::Rgb(0, 220, 120);
const NORMAL_FG: Color      = Color::Rgb(120, 120, 180);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_statusbar(frame, app, chunks[2]);

    if app.mode == Mode::Help {
        draw_help_overlay(frame, area);
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let file = std::path::Path::new(&app.data.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&app.data.file_path);

    let dirty_indicator = if app.dirty {
        Span::styled(" [+]", Style::default().fg(DIRTY_FG).bg(HEADER_BG).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("    ", Style::default().bg(HEADER_BG))
    };

    let sheet_tabs: Vec<Span> = app.data.sheets.iter().enumerate().map(|(i, s)| {
        if i == app.active_sheet {
            Span::styled(
                format!(" {} ", s.name),
                Style::default().fg(Color::Black).bg(Color::Rgb(140, 180, 255)).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!(" {} ", s.name), Style::default().fg(BORDER_FG).bg(HEADER_BG))
        }
    }).collect();

    let mut spans = vec![
        Span::styled(
            format!(" 📊 {} ", file),
            Style::default().fg(HEADER_FG).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ),
        dirty_indicator,
        Span::styled("  │  ", Style::default().fg(BORDER_FG).bg(HEADER_BG)),
    ];
    spans.extend(sheet_tabs);

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(HEADER_BG)),
        area,
    );
}

// ── Table ─────────────────────────────────────────────────────────────────────

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let visible_h = area.height.saturating_sub(3) as usize;
    let visible_w = area.width.saturating_sub(2) as usize;

    let col_widths = compute_col_widths(app, visible_w);
    sync_offsets(app, visible_h, &col_widths, visible_w);

    let col_offset = app.col_offset;
    let row_offset = app.row_offset;
    let in_insert  = app.mode == Mode::Insert;

    let sheet = app.current_sheet();
    if sheet.row_count() == 0 {
        frame.render_widget(
            Paragraph::new("(arquivo vazio)")
                .style(Style::default().fg(BORDER_FG))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER_FG))),
            area,
        );
        return;
    }

    let col_count  = sheet.col_count();
    let row_count  = sheet.row_count();
    let cursor_row = app.cursor_row;
    let cursor_col = app.cursor_col;
    let edit_buf   = app.edit_buffer.clone();
    let search_q   = app.search_query.clone();

    // header row (linha 0)
    let header_cells: Vec<RCell> = (col_offset..col_count)
        .map(|c| {
            let txt   = sheet.get(0, c).to_string();
            let width = col_widths.get(c).copied().unwrap_or(10) as usize;
            RCell::from(truncate(&txt, width))
                .style(Style::default().fg(HEADER_FG).bg(HEADER_BG).add_modifier(Modifier::BOLD))
        })
        .collect();

    let data_start = if row_count > 1 { 1 } else { 0 };

    let rows: Vec<Row> = (data_start..row_count)
        .skip(row_offset)
        .take(visible_h)
        .map(|r| {
            let is_cursor = r == cursor_row;
            let row_bg = if is_cursor { CURSOR_ROW_BG }
                         else if (r - data_start) % 2 == 0 { EVEN_BG }
                         else { ODD_BG };

            let cells: Vec<RCell> = (col_offset..col_count)
                .map(|c| {
                    let is_active = is_cursor && c == cursor_col;
                    let txt = if is_active && in_insert {
                        format!("{}█", edit_buf)
                    } else {
                        sheet.get(r, c).to_string()
                    };

                    let width   = col_widths.get(c).copied().unwrap_or(10) as usize;
                    let display = truncate(&txt, width);
                    let is_match = !search_q.is_empty()
                        && sheet.get(r, c).to_string().to_lowercase().contains(&search_q);

                    let style = if is_active && in_insert {
                        Style::default().bg(INSERT_CELL_BG).fg(INSERT_CELL_FG).add_modifier(Modifier::BOLD)
                    } else if is_active {
                        Style::default().bg(ACTIVE_CELL_BG).fg(ACTIVE_CELL_FG).add_modifier(Modifier::BOLD)
                    } else if is_match {
                        Style::default().bg(row_bg).fg(MATCH_FG).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().bg(row_bg).fg(Color::White)
                    };

                    RCell::from(display).style(style)
                })
                .collect();

            Row::new(cells).height(1).style(Style::default().bg(row_bg))
        })
        .collect();

    let constraints: Vec<Constraint> = (col_offset..col_count)
        .map(|c| Constraint::Length(col_widths.get(c).copied().unwrap_or(10)))
        .collect();

    let mut state = TableState::default();
    frame.render_stateful_widget(
        Table::new(rows, constraints)
            .header(Row::new(header_cells).height(1))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER_FG))),
        area,
        &mut state,
    );
}

// ── Status bar ────────────────────────────────────────────────────────────────

fn draw_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.mode {
        Mode::Search => {
            let hits = if app.search_hits.is_empty() {
                " nenhum resultado".to_string()
            } else {
                format!(" {}/{}", app.search_idx + 1, app.search_hits.len())
            };
            Line::from(vec![
                Span::styled(" / ", Style::default().fg(Color::Yellow).bg(SEARCH_BG).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}{}", app.search_query, hits), Style::default().fg(Color::White).bg(SEARCH_BG)),
            ])
        }

        Mode::Command => {
            Line::from(vec![
                Span::styled(" : ", Style::default().fg(Color::Yellow).bg(COMMAND_BG).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}█", app.command_buffer), Style::default().fg(Color::White).bg(COMMAND_BG)),
            ])
        }

        Mode::Insert => {
            let sheet    = app.current_sheet();
            let col_name = sheet.get(0, app.cursor_col).to_string();
            Line::from(vec![
                Span::styled(" -- INSERT -- ", Style::default().fg(INSERT_FG).bg(STATUS_BG).add_modifier(Modifier::BOLD)),
                Span::styled(" │ ", Style::default().fg(BORDER_FG).bg(STATUS_BG)),
                Span::styled(format!(" {} ", col_name), Style::default().fg(Color::Cyan).bg(STATUS_BG)),
                Span::styled(" │ ", Style::default().fg(BORDER_FG).bg(STATUS_BG)),
                Span::styled(format!(" {} ", app.edit_buffer), Style::default().fg(Color::White).bg(STATUS_BG)),
                Span::styled("  [Enter] confirmar  [Esc] cancelar", Style::default().fg(BORDER_FG).bg(STATUS_BG)),
            ])
        }

        _ => {
            let sheet    = app.current_sheet();
            let col_name = sheet.get(0, app.cursor_col).to_string();
            let cell_val = sheet.get(app.cursor_row, app.cursor_col).to_string();

            let mode_span = Span::styled(
                " -- NORMAL -- ",
                Style::default().fg(NORMAL_FG).bg(STATUS_BG),
            );

            // mensagem de status temporária (salvo, erro, etc.)
            let msg = if let Some(ref m) = app.status_msg {
                Span::styled(format!(" {} ", m), Style::default().fg(Color::Yellow).bg(STATUS_BG))
            } else {
                Span::styled(
                    "  [?] ajuda  [i] editar  [:] comando  [/] busca  [q] sair ",
                    Style::default().fg(BORDER_FG).bg(STATUS_BG),
                )
            };

            Line::from(vec![
                Span::styled(
                    format!(" Ln {}/{} ", app.cursor_row + 1, sheet.row_count()),
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" Col {}/{} ", app.cursor_col + 1, sheet.col_count()),
                    Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" │ ", Style::default().fg(BORDER_FG).bg(STATUS_BG)),
                Span::styled(
                    format!(" {} › {} ", col_name, cell_val),
                    Style::default().fg(Color::White).bg(STATUS_BG).add_modifier(Modifier::BOLD),
                ),
                mode_span,
                msg,
            ])
        }
    };

    frame.render_widget(Paragraph::new(content).style(Style::default().bg(STATUS_BG)), area);
}

// ── Help overlay ──────────────────────────────────────────────────────────────

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let width  = 52u16.min(area.width.saturating_sub(4));
    let height = 30u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup);

    let help_text = vec![
        Line::from(Span::styled(" Atalhos de Teclado", Style::default().fg(HEADER_FG).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(" Navegação", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("j/k"), desc(" linha abaixo / acima")]),
        Line::from(vec![key("h/l"), desc(" coluna esq / dir")]),
        Line::from(vec![key("gg"),  desc("  primeira linha")]),
        Line::from(vec![key("G"),   desc("   última linha")]),
        Line::from(vec![key("0/$"), desc(" primeira / última coluna")]),
        Line::from(vec![key("Tab"), desc(" próxima sheet")]),
        Line::from(""),
        Line::from(Span::styled(" Edição", Style::default().fg(INSERT_FG).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("i"),   desc("   editar célula")]),
        Line::from(vec![key("o/O"), desc(" nova linha abaixo / acima")]),
        Line::from(vec![key("dd"),  desc("  deletar linha")]),
        Line::from(vec![key("yy"),  desc("  copiar linha")]),
        Line::from(vec![key("p/P"), desc(" colar abaixo / acima")]),
        Line::from(vec![key("x"),   desc("   limpar célula")]),
        Line::from(vec![key("u"),   desc("   desfazer (undo)")]),
        Line::from(vec![key("^r"),  desc("  refazer (redo)")]),
        Line::from(vec![key("^s"),  desc("  salvar")]),
        Line::from(""),
        Line::from(Span::styled(" Comandos", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(vec![key(":w"),  desc("  salvar")]),
        Line::from(vec![key(":q"),  desc("  sair")]),
        Line::from(vec![key(":wq"), desc(" salvar e sair")]),
        Line::from(vec![key(":q!"), desc(" sair sem salvar")]),
        Line::from(""),
        Line::from(Span::styled(" Busca", Style::default().fg(MATCH_FG).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("/"),   desc("   buscar")]),
        Line::from(vec![key("n/N"), desc(" próx / ant resultado")]),
    ];

    frame.render_widget(
        Paragraph::new(help_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(140, 180, 255)))
                .style(Style::default().bg(Color::Rgb(15, 15, 30))),
        ),
        popup,
    );
}

fn key(k: &str) -> Span<'static> {
    Span::styled(format!(" {:5}", k), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
}

fn desc(d: &str) -> Span<'static> {
    Span::styled(d.to_string(), Style::default().fg(Color::White))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn compute_col_widths(app: &App, available_w: usize) -> Vec<u16> {
    let sheet      = app.current_sheet();
    let col_count  = sheet.col_count();
    let sample_rows = 50.min(sheet.row_count());

    let mut widths: Vec<usize> = (0..col_count).map(|c| {
        (0..sample_rows)
            .map(|r| display_width(&sheet.get(r, c).to_string()))
            .max()
            .unwrap_or(4)
            .clamp(4, 30)
    }).collect();

    let total: usize = widths.iter().sum::<usize>() + col_count;
    if total > available_w && col_count > 0 {
        let per_col = (available_w / col_count).clamp(6, 25);
        widths = widths.iter().map(|&w| w.min(per_col)).collect();
    }

    widths.iter().map(|&w| w as u16).collect()
}

fn sync_offsets(app: &mut App, visible_h: usize, col_widths: &[u16], visible_w: usize) {
    let data_row = app.cursor_row.saturating_sub(1);
    if data_row < app.row_offset {
        app.row_offset = data_row;
    } else if data_row >= app.row_offset + visible_h {
        app.row_offset = data_row + 1 - visible_h;
    }

    if app.cursor_col < app.col_offset {
        app.col_offset = app.cursor_col;
    } else {
        let mut used = 0usize;
        let mut last_visible = app.col_offset;
        for c in app.col_offset..col_widths.len() {
            used += col_widths[c] as usize + 1;
            if used > visible_w { break; }
            last_visible = c;
        }
        if app.cursor_col > last_visible {
            app.col_offset = app.cursor_col;
        }
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    let w = display_width(s);
    if w <= max_chars {
        format!("{:<width$}", s, width = max_chars)
    } else {
        let mut out = String::new();
        let mut used = 0;
        for ch in s.chars() {
            let cw = if (ch as u32) > 0x2E7F { 2 } else { 1 };
            if used + cw + 1 > max_chars { break; }
            out.push(ch);
            used += cw;
        }
        format!("{:<width$}…", out, width = max_chars.saturating_sub(1))
    }
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if (c as u32) > 0x2E7F { 2 } else { 1 }).sum()
}
