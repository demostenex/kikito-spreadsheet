use crate::table::{Cell, TableData};

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
    Help,
}

#[derive(Debug, Clone)]
pub enum Action {
    EditCell { sheet: usize, row: usize, col: usize, old: Cell, new: Cell },
    DeleteRow { sheet: usize, row: usize, cells: Vec<Cell> },
    InsertRow { sheet: usize, row: usize },
    ClearCell { sheet: usize, row: usize, col: usize, old: Cell },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    None,
    Save,
    Quit,
    SaveQuit,
    ForceQuit,
    Unknown(String),
}

#[derive(Debug)]
pub struct App {
    pub data:           TableData,
    pub active_sheet:   usize,
    pub cursor_row:     usize,
    pub cursor_col:     usize,
    pub row_offset:     usize,
    pub col_offset:     usize,
    pub mode:           Mode,
    pub pending_key:    Option<char>,
    pub edit_buffer:    String,
    pub command_buffer: String,
    pub search_query:   String,
    pub search_hits:    Vec<(usize, usize)>,
    pub search_idx:     usize,
    pub clipboard:      Option<Vec<Cell>>,
    pub dirty:          bool,
    pub undo_stack:     Vec<Action>,
    pub redo_stack:     Vec<Action>,
    pub should_quit:    bool,
    pub status_msg:     Option<String>,
}

impl App {
    pub fn new(data: TableData) -> Self {
        Self {
            data,
            active_sheet:   0,
            cursor_row:     0,
            cursor_col:     0,
            row_offset:     0,
            col_offset:     0,
            mode:           Mode::Normal,
            pending_key:    None,
            edit_buffer:    String::new(),
            command_buffer: String::new(),
            search_query:   String::new(),
            search_hits:    Vec::new(),
            search_idx:     0,
            clipboard:      None,
            dirty:          false,
            undo_stack:     Vec::new(),
            redo_stack:     Vec::new(),
            should_quit:    false,
            status_msg:     None,
        }
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    pub fn current_sheet(&self) -> &crate::table::Sheet {
        &self.data.sheets[self.active_sheet]
    }

    fn current_sheet_mut(&mut self) -> &mut crate::table::Sheet {
        &mut self.data.sheets[self.active_sheet]
    }

    // ── Navegação ────────────────────────────────────────────────────────────

    pub fn scroll_down(&mut self) {
        let max = self.current_sheet().row_count().saturating_sub(1);
        if self.cursor_row < max { self.cursor_row += 1; }
    }

    pub fn scroll_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        let max = self.current_sheet().col_count().saturating_sub(1);
        if self.cursor_col < max { self.cursor_col += 1; }
    }

    pub fn scroll_left(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(1);
    }

    pub fn goto_first_row(&mut self) {
        self.cursor_row = 0;
    }

    pub fn goto_last_row(&mut self) {
        self.cursor_row = self.current_sheet().row_count().saturating_sub(1);
    }

    pub fn goto_first_col(&mut self) {
        self.cursor_col = 0;
    }

    pub fn goto_last_col(&mut self) {
        self.cursor_col = self.current_sheet().col_count().saturating_sub(1);
    }

    pub fn next_sheet(&mut self) {
        self.active_sheet = (self.active_sheet + 1) % self.data.sheet_count();
        self.reset_cursor();
    }

    pub fn prev_sheet(&mut self) {
        let count = self.data.sheet_count();
        self.active_sheet = (self.active_sheet + count - 1) % count;
        self.reset_cursor();
    }

    // ── Pending key (sequências gg / dd / yy) ────────────────────────────────

    /// Retorna true se a sequência foi completada e consumida.
    pub fn handle_pending_key(&mut self, c: char) -> bool {
        if let Some(prev) = self.pending_key.take() {
            match (prev, c) {
                ('g', 'g') => { self.goto_first_row(); }
                ('d', 'd') => { self.delete_row(); }
                ('y', 'y') => { self.yank_row(); }
                _ => {}
            }
            true
        } else if matches!(c, 'g' | 'd' | 'y') {
            self.pending_key = Some(c);
            true
        } else {
            false
        }
    }

    // ── Modo Insert ──────────────────────────────────────────────────────────

    pub fn enter_insert(&mut self) {
        self.edit_buffer = self.current_sheet().get(self.cursor_row, self.cursor_col).to_string();
        self.mode = Mode::Insert;
        self.pending_key = None;
    }

    pub fn edit_push(&mut self, c: char) {
        self.edit_buffer.push(c);
    }

    pub fn edit_backspace(&mut self) {
        self.edit_buffer.pop();
    }

    pub fn confirm_edit(&mut self) {
        let new_cell = parse_cell(&self.edit_buffer);
        let old_cell = self.current_sheet().get(self.cursor_row, self.cursor_col).clone();
        let row = self.cursor_row;
        let col = self.cursor_col;
        let sheet = self.active_sheet;
        self.current_sheet_mut().set_cell(row, col, new_cell.clone());
        self.push_undo(Action::EditCell { sheet, row, col, old: old_cell, new: new_cell });
        self.dirty = true;
        self.mode = Mode::Normal;
    }

    pub fn cancel_edit(&mut self) {
        self.edit_buffer.clear();
        self.mode = Mode::Normal;
    }

    // ── Operações de linha ───────────────────────────────────────────────────

    pub fn insert_row_below(&mut self) {
        let row = self.cursor_row + 1;
        let sheet = self.active_sheet;
        let empty = self.current_sheet().empty_row();
        self.current_sheet_mut().insert_row(row, empty);
        self.cursor_row = row;
        self.push_undo(Action::InsertRow { sheet, row });
        self.dirty = true;
        self.enter_insert();
    }

    pub fn insert_row_above(&mut self) {
        let row = self.cursor_row;
        let sheet = self.active_sheet;
        let empty = self.current_sheet().empty_row();
        self.current_sheet_mut().insert_row(row, empty);
        self.push_undo(Action::InsertRow { sheet, row });
        self.dirty = true;
        self.enter_insert();
    }

    pub fn delete_row(&mut self) {
        let row = self.cursor_row;
        let sheet = self.active_sheet;
        if let Some(cells) = self.current_sheet_mut().delete_row(row) {
            self.push_undo(Action::DeleteRow { sheet, row, cells });
            self.dirty = true;
            let max = self.current_sheet().row_count().saturating_sub(1);
            if self.cursor_row > max { self.cursor_row = max; }
        }
    }

    pub fn yank_row(&mut self) {
        let row = self.cursor_row;
        let cells: Vec<Cell> = (0..self.current_sheet().col_count())
            .map(|c| self.current_sheet().get(row, c).clone())
            .collect();
        self.clipboard = Some(cells);
        self.status_msg = Some("Linha copiada".into());
    }

    pub fn paste_below(&mut self) {
        if let Some(cells) = self.clipboard.clone() {
            let row = self.cursor_row + 1;
            let sheet = self.active_sheet;
            self.current_sheet_mut().insert_row(row, cells);
            self.push_undo(Action::InsertRow { sheet, row });
            self.cursor_row = row;
            self.dirty = true;
        }
    }

    pub fn paste_above(&mut self) {
        if let Some(cells) = self.clipboard.clone() {
            let row = self.cursor_row;
            let sheet = self.active_sheet;
            self.current_sheet_mut().insert_row(row, cells);
            self.push_undo(Action::InsertRow { sheet, row });
            self.dirty = true;
        }
    }

    pub fn clear_cell(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let sheet = self.active_sheet;
        let old = self.current_sheet().get(row, col).clone();
        self.current_sheet_mut().set_cell(row, col, Cell::Empty);
        self.push_undo(Action::ClearCell { sheet, row, col, old });
        self.dirty = true;
    }

    // ── Undo / Redo ──────────────────────────────────────────────────────────

    fn push_undo(&mut self, action: Action) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            match action.clone() {
                Action::EditCell { sheet, row, col, old, .. } => {
                    self.data.sheets[sheet].set_cell(row, col, old);
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
                Action::DeleteRow { sheet, row, cells } => {
                    self.data.sheets[sheet].insert_row(row, cells);
                    self.cursor_row = row;
                }
                Action::InsertRow { sheet, row } => {
                    self.data.sheets[sheet].delete_row(row);
                    self.cursor_row = row.saturating_sub(1);
                }
                Action::ClearCell { sheet, row, col, old } => {
                    self.data.sheets[sheet].set_cell(row, col, old);
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
            }
            self.redo_stack.push(action);
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            match action.clone() {
                Action::EditCell { sheet, row, col, new, .. } => {
                    self.data.sheets[sheet].set_cell(row, col, new);
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
                Action::DeleteRow { sheet, row, .. } => {
                    self.data.sheets[sheet].delete_row(row);
                    self.cursor_row = row.saturating_sub(1);
                }
                Action::InsertRow { sheet, row } => {
                    let empty = self.data.sheets[sheet].empty_row();
                    self.data.sheets[sheet].insert_row(row, empty);
                    self.cursor_row = row;
                }
                Action::ClearCell { sheet, row, col, .. } => {
                    self.data.sheets[sheet].set_cell(row, col, Cell::Empty);
                }
            }
            self.undo_stack.push(action);
        }
    }

    // ── Modo Command ─────────────────────────────────────────────────────────

    pub fn enter_command(&mut self) {
        self.command_buffer.clear();
        self.mode = Mode::Command;
        self.pending_key = None;
    }

    pub fn command_push(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    pub fn execute_command(&mut self) -> CommandResult {
        let cmd = self.command_buffer.trim().to_string();
        self.command_buffer.clear();
        self.mode = Mode::Normal;
        match cmd.as_str() {
            "w"       => CommandResult::Save,
            "q"       => CommandResult::Quit,
            "wq" | "x" => CommandResult::SaveQuit,
            "q!"      => CommandResult::ForceQuit,
            other     => CommandResult::Unknown(other.to_owned()),
        }
    }

    pub fn cancel_command(&mut self) {
        self.command_buffer.clear();
        self.mode = Mode::Normal;
    }

    // ── Busca ────────────────────────────────────────────────────────────────

    pub fn enter_search(&mut self) {
        self.mode = Mode::Search;
        self.search_query.clear();
        self.search_hits.clear();
        self.search_idx = 0;
        self.pending_key = None;
    }

    pub fn update_search(&mut self, query: &str) {
        self.search_query = query.to_lowercase();
        let sheet = self.current_sheet();
        self.search_hits = (0..sheet.row_count())
            .flat_map(|r| (0..sheet.col_count()).map(move |c| (r, c)))
            .filter(|&(r, c)| {
                sheet.get(r, c).to_string().to_lowercase().contains(&self.search_query)
            })
            .collect();
        self.search_idx = 0;
        if let Some(&(r, c)) = self.search_hits.first() {
            self.cursor_row = r;
            self.cursor_col = c;
        }
    }

    pub fn exit_search(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn search_next(&mut self) {
        if self.search_hits.is_empty() { return; }
        self.search_idx = (self.search_idx + 1) % self.search_hits.len();
        let (r, c) = self.search_hits[self.search_idx];
        self.cursor_row = r;
        self.cursor_col = c;
    }

    pub fn search_prev(&mut self) {
        if self.search_hits.is_empty() { return; }
        let len = self.search_hits.len();
        self.search_idx = (self.search_idx + len - 1) % len;
        let (r, c) = self.search_hits[self.search_idx];
        self.cursor_row = r;
        self.cursor_col = c;
    }

    // ── Help ─────────────────────────────────────────────────────────────────

    pub fn toggle_help(&mut self) {
        self.mode = if self.mode == Mode::Help { Mode::Normal } else { Mode::Help };
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn reset_cursor(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.row_offset = 0;
        self.col_offset = 0;
    }
}

fn parse_cell(s: &str) -> Cell {
    if s.is_empty() { return Cell::Empty; }
    match s.to_uppercase().as_str() {
        "TRUE"  => return Cell::Bool(true),
        "FALSE" => return Cell::Bool(false),
        _ => {}
    }
    if let Ok(n) = s.parse::<f64>() { return Cell::Number(n); }
    Cell::Text(s.to_owned())
}

// ── Testes ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{Cell, Sheet, TableData};

    fn make_app(rows: usize, cols: usize) -> App {
        let rows_data: Vec<Vec<Cell>> = (0..rows)
            .map(|r| (0..cols).map(|c| Cell::Text(format!("r{}c{}", r, c))).collect())
            .collect();
        let sheet = Sheet::new("Sheet1", rows_data);
        App::new(TableData::new("test.xlsx", vec![sheet]))
    }

    fn make_multisheet_app() -> App {
        let s1 = Sheet::new("S1", vec![vec![Cell::Number(1.0)]]);
        let s2 = Sheet::new("S2", vec![vec![Cell::Number(2.0)]]);
        let s3 = Sheet::new("S3", vec![vec![Cell::Number(3.0)]]);
        App::new(TableData::new("test.xlsx", vec![s1, s2, s3]))
    }

    // ── Navegação ─────────────────────────────────────────────────────────────

    #[test]
    fn initial_cursor_is_zero() {
        let app = make_app(5, 5);
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn scroll_down_moves_cursor() {
        let mut app = make_app(5, 5);
        app.scroll_down();
        assert_eq!(app.cursor_row, 1);
    }

    #[test]
    fn scroll_down_clamps_at_last_row() {
        let mut app = make_app(3, 3);
        for _ in 0..10 { app.scroll_down(); }
        assert_eq!(app.cursor_row, 2);
    }

    #[test]
    fn scroll_up_clamps_at_zero() {
        let mut app = make_app(3, 3);
        app.scroll_up();
        assert_eq!(app.cursor_row, 0);
    }

    #[test]
    fn scroll_right_clamps_at_last_col() {
        let mut app = make_app(3, 3);
        for _ in 0..10 { app.scroll_right(); }
        assert_eq!(app.cursor_col, 2);
    }

    #[test]
    fn scroll_left_clamps_at_zero() {
        let mut app = make_app(3, 3);
        app.scroll_left();
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn goto_last_row() {
        let mut app = make_app(5, 5);
        app.goto_last_row();
        assert_eq!(app.cursor_row, 4);
    }

    #[test]
    fn goto_first_row() {
        let mut app = make_app(5, 5);
        app.goto_last_row();
        app.goto_first_row();
        assert_eq!(app.cursor_row, 0);
    }

    #[test]
    fn goto_first_col() {
        let mut app = make_app(5, 5);
        app.goto_last_col();
        app.goto_first_col();
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn next_sheet_advances() {
        let mut app = make_multisheet_app();
        app.next_sheet();
        assert_eq!(app.active_sheet, 1);
    }

    #[test]
    fn next_sheet_wraps_around() {
        let mut app = make_multisheet_app();
        app.active_sheet = 2;
        app.next_sheet();
        assert_eq!(app.active_sheet, 0);
    }

    #[test]
    fn prev_sheet_wraps_around() {
        let mut app = make_multisheet_app();
        app.prev_sheet();
        assert_eq!(app.active_sheet, 2);
    }

    #[test]
    fn next_sheet_resets_cursor() {
        let mut app = make_multisheet_app();
        app.cursor_row = 5;
        app.cursor_col = 3;
        app.next_sheet();
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 0);
    }

    // ── Pending key (gg / dd / yy) ────────────────────────────────────────────

    #[test]
    fn gg_goes_to_first_row() {
        let mut app = make_app(5, 5);
        app.goto_last_row();
        app.handle_pending_key('g');
        app.handle_pending_key('g');
        assert_eq!(app.cursor_row, 0);
    }

    #[test]
    fn single_g_sets_pending() {
        let mut app = make_app(5, 5);
        app.handle_pending_key('g');
        assert_eq!(app.pending_key, Some('g'));
    }

    // ── Insert mode ───────────────────────────────────────────────────────────

    #[test]
    fn insert_mode_prefills_buffer() {
        let mut app = make_app(3, 3);
        app.enter_insert();
        assert_eq!(app.edit_buffer, "r0c0");
        assert_eq!(app.mode, Mode::Insert);
    }

    #[test]
    fn insert_mode_confirm_changes_cell() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        app.cursor_col = 1;
        app.enter_insert();
        app.edit_buffer = "novo".to_string();
        app.confirm_edit();
        assert_eq!(app.current_sheet().get(1, 1), &Cell::Text("novo".into()));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.dirty);
    }

    #[test]
    fn insert_mode_confirm_parses_number() {
        let mut app = make_app(3, 3);
        app.enter_insert();
        app.edit_buffer = "42".to_string();
        app.confirm_edit();
        assert_eq!(app.current_sheet().get(0, 0), &Cell::Number(42.0));
    }

    #[test]
    fn insert_mode_cancel_restores_cell() {
        let mut app = make_app(3, 3);
        let original = app.current_sheet().get(0, 0).clone();
        app.enter_insert();
        app.edit_buffer = "descartado".to_string();
        app.cancel_edit();
        assert_eq!(app.current_sheet().get(0, 0), &original);
        assert_eq!(app.mode, Mode::Normal);
    }

    // ── Operações de linha ────────────────────────────────────────────────────

    #[test]
    fn dd_deletes_row() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        app.handle_pending_key('d');
        app.handle_pending_key('d');
        assert_eq!(app.current_sheet().row_count(), 2);
    }

    #[test]
    fn x_clears_cell() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        app.cursor_col = 1;
        app.clear_cell();
        assert_eq!(app.current_sheet().get(1, 1), &Cell::Empty);
        assert!(app.dirty);
    }

    #[test]
    fn yy_copies_row() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        app.handle_pending_key('y');
        app.handle_pending_key('y');
        assert!(app.clipboard.is_some());
        let clip = app.clipboard.as_ref().unwrap();
        assert_eq!(clip[0], Cell::Text("r1c0".into()));
    }

    #[test]
    fn p_pastes_row_below() {
        let mut app = make_app(3, 3);
        app.cursor_row = 0;
        app.yank_row();
        app.paste_below();
        assert_eq!(app.current_sheet().row_count(), 4);
        assert_eq!(app.cursor_row, 1);
    }

    #[test]
    fn p_paste_above() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        app.yank_row();
        app.paste_above();
        assert_eq!(app.current_sheet().row_count(), 4);
        assert_eq!(app.current_sheet().get(1, 0), &Cell::Text("r1c0".into()));
    }

    // ── Undo / Redo ───────────────────────────────────────────────────────────

    #[test]
    fn undo_reverts_edit() {
        let mut app = make_app(3, 3);
        let original = app.current_sheet().get(1, 1).clone();
        app.cursor_row = 1;
        app.cursor_col = 1;
        app.enter_insert();
        app.edit_buffer = "mudou".into();
        app.confirm_edit();
        app.undo();
        assert_eq!(app.current_sheet().get(1, 1), &original);
    }

    #[test]
    fn undo_reverts_delete() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        let original_cell = app.current_sheet().get(1, 0).clone();
        let count_before = app.current_sheet().row_count();
        app.delete_row();
        app.undo();
        assert_eq!(app.current_sheet().row_count(), count_before);
        assert_eq!(app.current_sheet().get(1, 0), &original_cell);
    }

    #[test]
    fn redo_reapplies_edit() {
        let mut app = make_app(3, 3);
        app.cursor_row = 1;
        app.cursor_col = 1;
        app.enter_insert();
        app.edit_buffer = "mudou".into();
        app.confirm_edit();
        app.undo();
        app.redo();
        assert_eq!(app.current_sheet().get(1, 1), &Cell::Text("mudou".into()));
    }

    #[test]
    fn dirty_flag_set_on_edit() {
        let mut app = make_app(3, 3);
        assert!(!app.dirty);
        app.enter_insert();
        app.edit_buffer = "x".into();
        app.confirm_edit();
        assert!(app.dirty);
    }

    #[test]
    fn dirty_flag_cleared_externally() {
        let mut app = make_app(3, 3);
        app.dirty = true;
        app.dirty = false;
        assert!(!app.dirty);
    }

    // ── Command mode ──────────────────────────────────────────────────────────

    #[test]
    fn execute_w_returns_save() {
        let mut app = make_app(3, 3);
        app.enter_command();
        app.command_buffer = "w".into();
        assert_eq!(app.execute_command(), CommandResult::Save);
    }

    #[test]
    fn execute_q_returns_quit() {
        let mut app = make_app(3, 3);
        app.enter_command();
        app.command_buffer = "q".into();
        assert_eq!(app.execute_command(), CommandResult::Quit);
    }

    #[test]
    fn execute_wq_returns_save_quit() {
        let mut app = make_app(3, 3);
        app.enter_command();
        app.command_buffer = "wq".into();
        assert_eq!(app.execute_command(), CommandResult::SaveQuit);
    }

    #[test]
    fn execute_q_bang_returns_force_quit() {
        let mut app = make_app(3, 3);
        app.enter_command();
        app.command_buffer = "q!".into();
        assert_eq!(app.execute_command(), CommandResult::ForceQuit);
    }

    // ── Busca ─────────────────────────────────────────────────────────────────

    #[test]
    fn search_filters_rows() {
        let rows = vec![
            vec![Cell::Text("Alice".into()), Cell::Number(30.0)],
            vec![Cell::Text("Bob".into()),   Cell::Number(25.0)],
            vec![Cell::Text("Alice2".into()), Cell::Number(20.0)],
        ];
        let mut app = App::new(TableData::new("t.csv", vec![Sheet::new("S", rows)]));
        app.enter_search();
        app.update_search("alice");
        assert_eq!(app.search_hits.len(), 2);
    }

    #[test]
    fn search_next_cycles_results() {
        let rows = vec![
            vec![Cell::Text("foo".into())],
            vec![Cell::Text("foo".into())],
        ];
        let mut app = App::new(TableData::new("t.csv", vec![Sheet::new("S", rows)]));
        app.enter_search();
        app.update_search("foo");
        app.search_next();
        assert_eq!(app.search_idx, 1);
        app.search_next();
        assert_eq!(app.search_idx, 0);
    }

    #[test]
    fn search_prev_cycles_results() {
        let rows = vec![
            vec![Cell::Text("foo".into())],
            vec![Cell::Text("foo".into())],
        ];
        let mut app = App::new(TableData::new("t.csv", vec![Sheet::new("S", rows)]));
        app.enter_search();
        app.update_search("foo");
        app.search_prev();
        assert_eq!(app.search_idx, 1);
    }
}
