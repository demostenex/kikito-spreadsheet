use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell as RCell, Clear, Paragraph, Row, Table, TableState},
};
use crate::app::{App, Mode};

const HEADER_BG: Color    = Color::Rgb(30, 30, 60);
const HEADER_FG: Color    = Color::Rgb(180, 180, 255);
const CURSOR_ROW_BG: Color = Color::Rgb(45, 75, 130);   // linha selecionada
const ACTIVE_CELL_BG: Color = Color::Rgb(255, 215, 0);  // célula ativa (amarelo ouro)
const ACTIVE_CELL_FG: Color = Color::Rgb(10, 10, 10);   // texto escuro na célula ativa
const EVEN_BG: Color      = Color::Rgb(18, 18, 28);
const ODD_BG: Color       = Color::Rgb(24, 24, 38);
const STATUS_BG: Color    = Color::Rgb(40, 40, 80);
const SEARCH_BG: Color    = Color::Rgb(60, 40, 0);
const MATCH_FG: Color     = Color::Rgb(255, 200, 0);
const BORDER_FG: Color    = Color::Rgb(80, 80, 120);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // header
            Constraint::Min(0),     // table
            Constraint::Length(1),  // status / search bar
        ])
        .split(area);

    draw_header(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_statusbar(frame, app, chunks[2]);

    if app.mode == Mode::Help {
        draw_help_overlay(frame, area);
    }
}

// ── Header ──────────────────────────────────────────────────────────────────

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let file = std::path::Path::new(&app.data.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&app.data.file_path);

    let sheet_tabs: Vec<Span> = app.data.sheets.iter().enumerate().map(|(i, s)| {
        if i == app.active_sheet {
            Span::styled(
                format!(" {} ", s.name),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(140, 180, 255))
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                format!(" {} ", s.name),
                Style::default().fg(BORDER_FG).bg(HEADER_BG),
            )
        }
    }).collect();

    let mut spans = vec![
        Span::styled(
            format!(" 📊 {} ", file),
            Style::default().fg(HEADER_FG).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(BORDER_FG).bg(HEADER_BG)),
    ];
    spans.extend(sheet_tabs);

    let header = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(HEADER_BG));
    frame.render_widget(header, area);
}

// ── Table ───────────────────────────────────────────────────────────────────

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let visible_h = area.height.saturating_sub(3) as usize;
    let visible_w = area.width.saturating_sub(2) as usize;

    // compute widths and sync offsets before immutable borrow of sheet
    let col_widths = compute_col_widths(app, visible_w);
    sync_offsets(app, visible_h, &col_widths, visible_w);

    let col_offset = app.col_offset;
    let row_offset = app.row_offset;

    let sheet = app.current_sheet();
    if sheet.row_count() == 0 {
        let empty = Paragraph::new("(arquivo vazio)")
            .style(Style::default().fg(BORDER_FG))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER_FG)));
        frame.render_widget(empty, area);
        return;
    }

    let col_count = sheet.col_count();
    let row_count = sheet.row_count();

    // header row (row 0)
    let header_cells: Vec<RCell> = (col_offset..col_count)
        .map(|c| {
            let txt = sheet.get(0, c).to_string();
            let width = col_widths.get(c).copied().unwrap_or(10) as usize;
            RCell::from(truncate(&txt, width))
                .style(Style::default().fg(HEADER_FG).bg(HEADER_BG).add_modifier(Modifier::BOLD))
        })
        .collect();

    let header_row = Row::new(header_cells).height(1);

    // data rows (skip row 0 which is header)
    let data_start = if row_count > 1 { 1 } else { 0 };
    let rows: Vec<Row> = (data_start..row_count)
        .skip(row_offset)
        .take(visible_h)
        .map(|r| {
            let is_cursor = r == app.cursor_row;
            let row_bg = if is_cursor {
                CURSOR_ROW_BG
            } else if (r - data_start) % 2 == 0 {
                EVEN_BG
            } else {
                ODD_BG
            };

            let cells: Vec<RCell> = (col_offset..col_count)
                .map(|c| {
                    let txt = sheet.get(r, c).to_string();
                    let width = col_widths.get(c).copied().unwrap_or(10) as usize;
                    let display = truncate(&txt, width);

                    let is_match = !app.search_query.is_empty()
                        && txt.to_lowercase().contains(&app.search_query);

                    let style = if is_cursor && c == app.cursor_col {
                        Style::default()
                            .bg(ACTIVE_CELL_BG)
                            .fg(ACTIVE_CELL_FG)
                            .add_modifier(Modifier::BOLD)
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

    let table = Table::new(rows, constraints)
        .header(header_row)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_FG)),
        );

    let mut state = TableState::default();
    frame.render_stateful_widget(table, area, &mut state);
}

// ── Status bar ──────────────────────────────────────────────────────────────

fn draw_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let sheet = app.current_sheet();

    let content = if app.mode == Mode::Search {
        let hits = if app.search_hits.is_empty() {
            " nenhum resultado".to_string()
        } else {
            format!(" {}/{}", app.search_idx + 1, app.search_hits.len())
        };
        Line::from(vec![
            Span::styled(" / ", Style::default().fg(Color::Yellow).bg(SEARCH_BG).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}{}", app.search_query, hits),
                Style::default().fg(Color::White).bg(SEARCH_BG),
            ),
        ])
    } else {
        let col_name  = sheet.get(0, app.cursor_col).to_string();
        let cell_val  = sheet.get(app.cursor_row, app.cursor_col).to_string();

        let row_info  = format!(" Ln {}/{} ", app.cursor_row + 1, sheet.row_count());
        let col_info  = format!(" Col {}/{} ", app.cursor_col + 1, sheet.col_count());
        let cell_ref  = format!(" {} › {} ", col_name, cell_val);
        let mode_info = "  [?] ajuda  [q] sair  [/] busca  [Tab] sheet ".to_string();
        Line::from(vec![
            Span::styled(row_info,  Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(col_info,  Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ",     Style::default().fg(BORDER_FG).bg(STATUS_BG)),
            Span::styled(cell_ref,  Style::default().fg(Color::White).bg(STATUS_BG).add_modifier(Modifier::BOLD)),
            Span::styled(mode_info, Style::default().fg(BORDER_FG).bg(STATUS_BG)),
        ])
    };

    let status = Paragraph::new(content).style(Style::default().bg(STATUS_BG));
    frame.render_widget(status, area);
}

// ── Help overlay ─────────────────────────────────────────────────────────────

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let width  = 46u16.min(area.width.saturating_sub(4));
    let height = 22u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup);

    let help_text = vec![
        Line::from(Span::styled(" Atalhos de Teclado", Style::default().fg(HEADER_FG).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![key("j / ↓"), desc("  Linha abaixo")]),
        Line::from(vec![key("k / ↑"), desc("  Linha acima")]),
        Line::from(vec![key("h / ←"), desc("  Coluna esquerda")]),
        Line::from(vec![key("l / →"), desc("  Coluna direita")]),
        Line::from(""),
        Line::from(vec![key("g"),     desc("      Primeira linha")]),
        Line::from(vec![key("G"),     desc("      Última linha")]),
        Line::from(vec![key("0"),     desc("      Primeira coluna")]),
        Line::from(vec![key("$"),     desc("      Última coluna")]),
        Line::from(""),
        Line::from(vec![key("Tab"),   desc("    Próxima sheet")]),
        Line::from(vec![key("S-Tab"), desc("  Sheet anterior")]),
        Line::from(""),
        Line::from(vec![key("/"),     desc("      Buscar")]),
        Line::from(vec![key("n / N"), desc("  Próx / Ant resultado")]),
        Line::from(vec![key("Esc"),   desc("    Sair da busca")]),
        Line::from(""),
        Line::from(vec![key("?"),     desc("      Toggle ajuda")]),
        Line::from(vec![key("q"),     desc("      Sair")]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(140, 180, 255)))
        .style(Style::default().bg(Color::Rgb(15, 15, 30)));

    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, popup);
}

fn key(k: &str) -> Span<'static> {
    Span::styled(
        format!(" {:5}", k),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )
}

fn desc(d: &str) -> Span<'static> {
    Span::styled(d.to_string(), Style::default().fg(Color::White))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn compute_col_widths(app: &App, available_w: usize) -> Vec<u16> {
    let sheet     = app.current_sheet();
    let col_count = sheet.col_count();
    let sample_rows = 50.min(sheet.row_count());

    let mut widths: Vec<usize> = (0..col_count).map(|c| {
        (0..sample_rows)
            .map(|r| display_width(&sheet.get(r, c).to_string()))
            .max()
            .unwrap_or(4)
            .clamp(4, 30)
    }).collect();

    // if all columns fit, keep them; otherwise distribute available width
    let total: usize = widths.iter().sum::<usize>() + col_count; // +1 per separator
    if total > available_w && col_count > 0 {
        let per_col = (available_w / col_count).clamp(6, 25);
        widths = widths.iter().map(|&w| w.min(per_col)).collect();
    }

    widths.iter().map(|&w| w as u16).collect()
}

fn sync_offsets(app: &mut App, visible_h: usize, col_widths: &[u16], visible_w: usize) {
    // vertical: keep cursor_row within [row_offset, row_offset + visible_h)
    // row 0 is header, data rows start at 1
    let data_row = app.cursor_row.saturating_sub(1);
    if data_row < app.row_offset {
        app.row_offset = data_row;
    } else if data_row >= app.row_offset + visible_h {
        app.row_offset = data_row + 1 - visible_h;
    }

    // horizontal: keep cursor_col within visible window
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
