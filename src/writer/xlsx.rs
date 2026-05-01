use std::path::Path;
use rust_xlsxwriter::{Workbook, XlsxError};
use crate::error::AppError;
use crate::table::{Cell, TableData};
use super::Writer;

pub struct XlsxWriter;

impl From<XlsxError> for AppError {
    fn from(e: XlsxError) -> Self {
        AppError::Parse(e.to_string())
    }
}

impl Writer for XlsxWriter {
    fn write(&self, data: &TableData, path: &Path) -> Result<(), AppError> {
        let mut workbook = Workbook::new();

        for sheet_data in &data.sheets {
            let ws = workbook.add_worksheet();
            ws.set_name(&sheet_data.name)?;

            for (r, row) in sheet_data.rows.iter().enumerate() {
                for (c, cell) in row.iter().enumerate() {
                    let row_u32 = r as u32;
                    let col_u16 = c as u16;
                    match cell {
                        Cell::Text(s)   => { ws.write_string(row_u32, col_u16, s)?; }
                        Cell::Number(n) => { ws.write_number(row_u32, col_u16, *n)?; }
                        Cell::Bool(b)   => { ws.write_boolean(row_u32, col_u16, *b)?; }
                        Cell::Empty     => {}
                    }
                }
            }
        }

        workbook.save(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::xlsx::XlsxReader;
    use crate::reader::Reader;
    use crate::table::{Cell, Sheet, TableData};
    use tempfile::NamedTempFile;

    fn sample() -> TableData {
        TableData::new("t.xlsx", vec![Sheet::new("Dados", vec![
            vec![Cell::Text("Nome".into()), Cell::Number(42.0), Cell::Bool(true)],
            vec![Cell::Text("Alice".into()), Cell::Number(30.0), Cell::Bool(false)],
        ])])
    }

    #[test]
    fn writes_single_sheet() {
        let f = NamedTempFile::with_suffix(".xlsx").unwrap();
        XlsxWriter.write(&sample(), f.path()).unwrap();
        assert!(f.path().exists());
    }

    #[test]
    fn roundtrip_xlsx() {
        let f = NamedTempFile::with_suffix(".xlsx").unwrap();
        XlsxWriter.write(&sample(), f.path()).unwrap();
        let loaded = XlsxReader.read(f.path()).unwrap();
        let s = loaded.first_sheet().unwrap();
        assert_eq!(s.get(0, 0), &Cell::Text("Nome".into()));
        assert_eq!(s.get(1, 1), &Cell::Number(30.0));
    }
}
