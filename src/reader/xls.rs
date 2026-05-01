use std::path::Path;
use calamine::{Reader as CalReader, open_workbook, Xls, Data, XlsError};
use crate::error::AppError;
use crate::table::{Cell, Sheet, TableData};
use super::Reader;

pub struct XlsReader;

fn data_to_cell(d: &Data) -> Cell {
    match d {
        Data::String(s) | Data::DateTimeIso(s) => {
            if s.is_empty() { Cell::Empty } else { Cell::Text(s.clone()) }
        }
        Data::Float(n)    => Cell::Number(*n),
        Data::Int(n)      => Cell::Number(*n as f64),
        Data::Bool(b)     => Cell::Bool(*b),
        Data::DateTime(dt) => Cell::Number(dt.as_f64()),
        Data::Error(_)    => Cell::Empty,
        Data::Empty       => Cell::Empty,
        _                 => Cell::Empty,
    }
}

impl Reader for XlsReader {
    fn read(&self, path: &Path) -> Result<TableData, AppError> {
        let mut workbook: Xls<_> = open_workbook(path)
            .map_err(|e: XlsError| AppError::Parse(e.to_string()))?;

        let names: Vec<String> = workbook.sheet_names().to_vec();

        let sheets = names
            .iter()
            .filter_map(|name| {
                workbook.worksheet_range(name).ok().map(|range| {
                    let rows = range
                        .rows()
                        .map(|row| row.iter().map(data_to_cell).collect())
                        .collect();
                    Sheet::new(name.clone(), rows)
                })
            })
            .collect();

        Ok(TableData::new(path.display().to_string(), sheets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
        }

    #[test]
    fn reads_legacy_xls() {
        let td = XlsReader.read(&fixture("sample.xls")).unwrap();
        assert!(!td.sheets.is_empty());
        assert!(td.first_sheet().unwrap().row_count() > 0);
    }
}
