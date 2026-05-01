pub mod csv;
pub mod xlsx;

use std::path::Path;
use crate::error::AppError;
use crate::table::TableData;

pub trait Writer {
    fn write(&self, data: &TableData, path: &Path) -> Result<(), AppError>;
}

pub fn writer_for(path: &Path) -> Result<Box<dyn Writer>, AppError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("csv")        => Ok(Box::new(csv::CsvWriter)),
        Some("xlsx")       => Ok(Box::new(xlsx::XlsxWriter)),
        Some("xls")        => Ok(Box::new(xlsx::XlsxWriter)), // salva como xlsx
        Some(e)            => Err(AppError::UnsupportedFormat(e.to_owned())),
        None               => Err(AppError::UnsupportedFormat(String::new())),
    }
}
