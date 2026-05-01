use std::path::Path;
use crate::error::AppError;
use crate::table::{Cell, TableData};
use super::Writer;

pub struct CsvWriter;

impl Writer for CsvWriter {
    fn write(&self, data: &TableData, path: &Path) -> Result<(), AppError> {
        let sheet = data.sheets.first().ok_or_else(|| AppError::Parse("sem sheets".into()))?;
        let mut wtr = ::csv::WriterBuilder::new()
            .delimiter(b',')
            .from_path(path)
            .map_err(|e| AppError::Parse(e.to_string()))?;

        for row in &sheet.rows {
            let record: Vec<String> = row.iter().map(Cell::to_string).collect();
            wtr.write_record(&record).map_err(|e| AppError::Parse(e.to_string()))?;
        }
        wtr.flush().map_err(AppError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::csv::CsvReader;
    use crate::reader::Reader;
    use crate::table::{Cell, Sheet, TableData};
    use tempfile::NamedTempFile;

    fn sample() -> TableData {
        TableData::new("t.csv", vec![Sheet::new("S", vec![
            vec![Cell::Text("Nome".into()), Cell::Text("Idade".into())],
            vec![Cell::Text("Alice".into()), Cell::Number(30.0)],
        ])])
    }

    #[test]
    fn writes_simple_csv() {
        let f = NamedTempFile::with_suffix(".csv").unwrap();
        CsvWriter.write(&sample(), f.path()).unwrap();
        let content = std::fs::read_to_string(f.path()).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("30"));
    }

    #[test]
    fn roundtrip_csv() {
        let f = NamedTempFile::with_suffix(".csv").unwrap();
        let original = sample();
        CsvWriter.write(&original, f.path()).unwrap();
        let loaded = CsvReader.read(f.path()).unwrap();
        let s = loaded.first_sheet().unwrap();
        assert_eq!(s.get(1, 0), &Cell::Text("Alice".into()));
        assert_eq!(s.get(1, 1), &Cell::Number(30.0));
    }
}
