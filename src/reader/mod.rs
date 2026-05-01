pub mod csv;
pub mod xlsx;
pub mod xls;

use std::path::Path;
use crate::error::AppError;
use crate::table::TableData;

pub trait Reader {
    fn read(&self, path: &Path) -> Result<TableData, AppError>;
}

pub fn reader_for(path: &Path) -> Result<Box<dyn Reader>, AppError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("csv")           => Ok(Box::new(csv::CsvReader)),
        Some("xlsx")          => Ok(Box::new(xlsx::XlsxReader)),
        Some("xls")           => Ok(Box::new(xls::XlsReader)),
        Some(e)               => Err(AppError::UnsupportedFormat(e.to_owned())),
        None                  => Err(AppError::UnsupportedFormat(String::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn factory_csv() {
        assert!(reader_for(Path::new("data.csv")).is_ok());
    }

    #[test]
    fn factory_xlsx() {
        assert!(reader_for(Path::new("data.xlsx")).is_ok());
    }

    #[test]
    fn factory_xls() {
        assert!(reader_for(Path::new("data.xls")).is_ok());
    }

    #[test]
    fn factory_unknown_extension_error() {
        let result = reader_for(Path::new("data.ods"));
        assert!(matches!(result, Err(AppError::UnsupportedFormat(_))));
    }

    #[test]
    fn factory_no_extension_error() {
        let result = reader_for(Path::new("data"));
        assert!(matches!(result, Err(AppError::UnsupportedFormat(_))));
    }
}
