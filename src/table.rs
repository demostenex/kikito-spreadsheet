use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Text(String),
    Number(f64),
    Bool(bool),
    Empty,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Text(s)   => write!(f, "{}", s),
            Cell::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Cell::Bool(b)   => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            Cell::Empty     => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<Cell>>,
}

impl Sheet {
    pub fn new(name: impl Into<String>, rows: Vec<Vec<Cell>>) -> Self {
        Self { name: name.into(), rows }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn col_count(&self) -> usize {
        self.rows.iter().map(|r| r.len()).max().unwrap_or(0)
    }

    pub fn get(&self, row: usize, col: usize) -> &Cell {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .unwrap_or(&Cell::Empty)
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        if let Some(r) = self.rows.get_mut(row) {
            if col < r.len() {
                r[col] = cell;
            }
        }
    }

    pub fn delete_row(&mut self, row: usize) -> Option<Vec<Cell>> {
        if row < self.rows.len() {
            Some(self.rows.remove(row))
        } else {
            None
        }
    }

    pub fn insert_row(&mut self, row: usize, cells: Vec<Cell>) {
        let col_count = self.col_count();
        let mut padded = cells;
        while padded.len() < col_count { padded.push(Cell::Empty); }
        self.rows.insert(row.min(self.rows.len()), padded);
    }

    pub fn empty_row(&self) -> Vec<Cell> {
        vec![Cell::Empty; self.col_count()]
    }
}

#[derive(Debug, Clone)]
pub struct TableData {
    pub sheets:    Vec<Sheet>,
    pub file_path: String,
}

impl TableData {
    pub fn new(file_path: impl Into<String>, sheets: Vec<Sheet>) -> Self {
        Self { file_path: file_path.into(), sheets }
    }

    pub fn first_sheet(&self) -> Option<&Sheet> {
        self.sheets.first()
    }

    pub fn sheet_by_name(&self, name: &str) -> Option<&Sheet> {
        self.sheets.iter().find(|s| s.name == name)
    }

    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Cell::Display ---

    #[test]
    fn cell_display_string() {
        assert_eq!(Cell::Text("hello".into()).to_string(), "hello");
    }

    #[test]
    fn cell_display_number_integer() {
        assert_eq!(Cell::Number(42.0).to_string(), "42");
    }

    #[test]
    fn cell_display_number_float() {
        assert_eq!(Cell::Number(3.14).to_string(), "3.14");
    }

    #[test]
    fn cell_display_bool_true() {
        assert_eq!(Cell::Bool(true).to_string(), "TRUE");
    }

    #[test]
    fn cell_display_bool_false() {
        assert_eq!(Cell::Bool(false).to_string(), "FALSE");
    }

    #[test]
    fn cell_display_empty() {
        assert_eq!(Cell::Empty.to_string(), "");
    }

    // --- Sheet ---

    fn sample_sheet() -> Sheet {
        Sheet::new("Sheet1", vec![
            vec![Cell::Text("Name".into()), Cell::Text("Age".into())],
            vec![Cell::Text("Alice".into()), Cell::Number(30.0)],
            vec![Cell::Text("Bob".into()),   Cell::Number(25.0)],
        ])
    }

    #[test]
    fn sheet_row_count() {
        assert_eq!(sample_sheet().row_count(), 3);
    }

    #[test]
    fn sheet_col_count() {
        assert_eq!(sample_sheet().col_count(), 2);
    }

    #[test]
    fn sheet_col_count_empty() {
        assert_eq!(Sheet::new("empty", vec![]).col_count(), 0);
    }

    #[test]
    fn sheet_get_existing_cell() {
        let sheet = sample_sheet();
        assert_eq!(sheet.get(1, 0), &Cell::Text("Alice".into()));
    }

    #[test]
    fn sheet_get_out_of_bounds_returns_empty() {
        let sheet = sample_sheet();
        assert_eq!(sheet.get(99, 99), &Cell::Empty);
    }

    // --- TableData ---

    fn sample_table() -> TableData {
        TableData::new("file.xlsx", vec![
            Sheet::new("Dados",    vec![vec![Cell::Number(1.0)]]),
            Sheet::new("Resumo",   vec![vec![Cell::Text("ok".into())]]),
        ])
    }

    #[test]
    fn tabledata_first_sheet() {
        let t = sample_table();
        assert_eq!(t.first_sheet().unwrap().name, "Dados");
    }

    #[test]
    fn tabledata_sheet_by_name_found() {
        let t = sample_table();
        assert!(t.sheet_by_name("Resumo").is_some());
    }

    #[test]
    fn tabledata_sheet_by_name_not_found() {
        let t = sample_table();
        assert!(t.sheet_by_name("Inexistente").is_none());
    }

    #[test]
    fn tabledata_sheet_count() {
        assert_eq!(sample_table().sheet_count(), 2);
    }

    #[test]
    fn tabledata_file_path_stored() {
        assert_eq!(sample_table().file_path, "file.xlsx");
    }

    // --- Sheet mutação ---

    #[test]
    fn set_cell_updates_value() {
        let mut s = sample_sheet();
        s.set_cell(1, 0, Cell::Text("Zed".into()));
        assert_eq!(s.get(1, 0), &Cell::Text("Zed".into()));
    }

    #[test]
    fn set_cell_out_of_bounds_does_nothing() {
        let mut s = sample_sheet();
        s.set_cell(99, 99, Cell::Number(1.0));
        assert_eq!(s.row_count(), 3);
    }

    #[test]
    fn delete_row_removes_row() {
        let mut s = sample_sheet();
        s.delete_row(1);
        assert_eq!(s.row_count(), 2);
        assert_eq!(s.get(1, 0), &Cell::Text("Bob".into()));
    }

    #[test]
    fn delete_row_returns_cells() {
        let mut s = sample_sheet();
        let removed = s.delete_row(1).unwrap();
        assert_eq!(removed[0], Cell::Text("Alice".into()));
    }

    #[test]
    fn insert_row_adds_at_position() {
        let mut s = sample_sheet();
        s.insert_row(1, vec![Cell::Text("Zed".into()), Cell::Number(99.0)]);
        assert_eq!(s.row_count(), 4);
        assert_eq!(s.get(1, 0), &Cell::Text("Zed".into()));
    }

    #[test]
    fn empty_row_has_correct_col_count() {
        let s = sample_sheet();
        assert_eq!(s.empty_row().len(), s.col_count());
    }
}
