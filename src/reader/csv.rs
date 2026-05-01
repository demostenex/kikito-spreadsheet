use std::path::Path;
use crate::error::AppError;
use crate::table::{Cell, Sheet, TableData};
use super::Reader;

pub struct CsvReader;

impl CsvReader {
    fn detect_delimiter(sample: &str) -> u8 {
        let counts = [
            (b',',  sample.chars().filter(|&c| c == ',').count()),
            (b';',  sample.chars().filter(|&c| c == ';').count()),
            (b'\t', sample.chars().filter(|&c| c == '\t').count()),
        ];
        counts.into_iter().max_by_key(|&(_, n)| n).map(|(d, _)| d).unwrap_or(b',')
    }
}

impl Reader for CsvReader {
    fn read(&self, path: &Path) -> Result<TableData, AppError> {
        let raw = std::fs::read(path).map_err(AppError::Io)?;

        // strip UTF-8 BOM if present
        let content = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
            std::str::from_utf8(&raw[3..]).map_err(|e| AppError::Parse(e.to_string()))?.to_owned()
        } else {
            String::from_utf8(raw).map_err(|e| AppError::Parse(e.to_string()))?
        };

        let delimiter = Self::detect_delimiter(&content);

        let mut rdr = ::csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .from_reader(content.as_bytes());

        let rows: Vec<Vec<Cell>> = rdr
            .records()
            .map(|r| {
                let record = r.map_err(|e| AppError::Parse(e.to_string()))?;
                Ok(record.iter().map(|f| {
                    if f.is_empty() {
                        Cell::Empty
                    } else if let Ok(n) = f.parse::<f64>() {
                        Cell::Number(n)
                    } else {
                        Cell::Text(f.to_owned())
                    }
                }).collect())
            })
            .collect::<Result<_, AppError>>()?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sheet")
            .to_owned();

        let sheet = Sheet::new(name, rows);
        let file_path = path.display().to_string();
        Ok(TableData::new(file_path, vec![sheet]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn read_str(content: &[u8]) -> Result<TableData, AppError> {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        f.write_all(content).unwrap();
        CsvReader.read(f.path())
    }

    #[test]
    fn reads_simple_csv() {
        let td = read_str(b"name,age\nAlice,30\nBob,25").unwrap();
        let sheet = td.first_sheet().unwrap();
        assert_eq!(sheet.row_count(), 3);
        assert_eq!(sheet.col_count(), 2);
    }

    #[test]
    fn reads_semicolon_delimited() {
        let td = read_str(b"a;b;c\n1;2;3").unwrap();
        let sheet = td.first_sheet().unwrap();
        assert_eq!(sheet.col_count(), 3);
    }

    #[test]
    fn reads_tab_delimited() {
        let td = read_str(b"a\tb\tc\n1\t2\t3").unwrap();
        let sheet = td.first_sheet().unwrap();
        assert_eq!(sheet.col_count(), 3);
    }

    #[test]
    fn reads_empty_csv() {
        let td = read_str(b"").unwrap();
        assert_eq!(td.first_sheet().unwrap().row_count(), 0);
    }

    #[test]
    fn reads_utf8_with_bom() {
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"nome,valor\nTeste,42");
        let td = read_str(&content).unwrap();
        let sheet = td.first_sheet().unwrap();
        assert_eq!(sheet.row_count(), 2);
    }

    #[test]
    fn parses_numbers() {
        let td = read_str(b"x\n3.14\n100").unwrap();
        let sheet = td.first_sheet().unwrap();
        assert_eq!(sheet.get(1, 0), &Cell::Number(3.14));
        assert_eq!(sheet.get(2, 0), &Cell::Number(100.0));
    }

    #[test]
    fn parses_empty_fields_as_empty_cell() {
        let td = read_str(b"a,b\n,2").unwrap();
        let sheet = td.first_sheet().unwrap();
        assert_eq!(sheet.get(1, 0), &Cell::Empty);
    }
}
