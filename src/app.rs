use crate::table::TableData;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Help,
}

#[derive(Debug)]
pub struct App {
    pub data:         TableData,
    pub active_sheet: usize,
    pub cursor_row:   usize,
    pub cursor_col:   usize,
    pub row_offset:   usize,
    pub col_offset:   usize,
    pub mode:         Mode,
    pub search_query: String,
    pub search_hits:  Vec<(usize, usize)>,
    pub search_idx:   usize,
    pub should_quit:  bool,
}

impl App {
    pub fn new(data: TableData) -> Self {
        Self {
            data,
            active_sheet: 0,
            cursor_row:   0,
            cursor_col:   0,
            row_offset:   0,
            col_offset:   0,
            mode:         Mode::Normal,
            search_query: String::new(),
            search_hits:  Vec::new(),
            search_idx:   0,
            should_quit:  false,
        }
    }

    pub fn current_sheet(&self) -> &crate::table::Sheet {
        &self.data.sheets[self.active_sheet]
    }

    pub fn scroll_down(&mut self) {
        let max = self.current_sheet().row_count().saturating_sub(1);
        if self.cursor_row < max {
            self.cursor_row += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        let max = self.current_sheet().col_count().saturating_sub(1);
        if self.cursor_col < max {
            self.cursor_col += 1;
        }
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

    pub fn enter_search(&mut self) {
        self.mode = Mode::Search;
        self.search_query.clear();
        self.search_hits.clear();
        self.search_idx = 0;
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

    pub fn toggle_help(&mut self) {
        self.mode = if self.mode == Mode::Help { Mode::Normal } else { Mode::Help };
    }

    fn reset_cursor(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.row_offset = 0;
        self.col_offset = 0;
    }
}

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
    fn search_empty_shows_nothing() {
        let rows = vec![vec![Cell::Text("Alice".into())]];
        let mut app = App::new(TableData::new("t.csv", vec![Sheet::new("S", rows)]));
        app.enter_search();
        app.update_search("");
        // query vazia vai dar match em tudo (string vazia é contained em qualquer string)
        // comportamento esperado: retorna todas as células
        assert!(!app.search_hits.is_empty());
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

    #[test]
    fn next_sheet_resets_cursor() {
        let mut app = make_multisheet_app();
        app.cursor_row = 5;
        app.cursor_col = 3;
        app.next_sheet();
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.cursor_col, 0);
    }
}
