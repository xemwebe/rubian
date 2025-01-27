use crate::blob::{Blob, BlobError};
use crate::table::{Row, RowAction, Table, TableType};
use core::fmt::Write;
use thiserror::Error;

type Result<T> = std::result::Result<T, HexError>;

#[derive(Error, Debug, Clone)]
pub enum HexError {
    #[error("formatting error")]
    Formatting(#[from] std::fmt::Error),
    #[error("binary corrupted")]
    BlobCorrupted(#[from] BlobError),
}

pub struct HexBinary {
    blob: Blob,
}

impl HexBinary {
    pub fn new(blob: Blob) -> Self {
        Self { blob }
    }

    pub fn as_hex_table(&self, bytes_per_line: u32) -> Result<Table> {
        prepare_hex_table(&self.blob, bytes_per_line)
    }

    pub fn header_info(&self) -> Vec<(String, String)> {
        vec![("Ident".to_string(), "Unknown file".to_string())]
    }
}

fn prepare_hex_table(blob: &Blob, bytes_per_line: u32) -> Result<Table> {
    let bytes_per_line = bytes_per_line as usize;
    let mut rows = Vec::new();
    let mut offset = 0;
    let max = blob.len();
    while offset < max {
        let mut first = true;
        let mut bytes = String::with_capacity(bytes_per_line * 3 - 1);
        let mut ascii_bytes = String::with_capacity(bytes_per_line);
        for idx in offset..max.min(offset + bytes_per_line) {
            let b = blob.get_u8(idx)?;
            if b >= 32 && b < 128 {
                write!(ascii_bytes, "{}", b as char)?;
            } else {
                write!(ascii_bytes, ".")?;
            };
            if first {
                write!(bytes, "{b:02X}")?;
                first = false;
            } else {
                write!(bytes, " {b:02X}")?;
            }
        }
        let offset_string = format!("0x{offset:#016x}");
        rows.push(Row {
            action: RowAction::None,
            content: vec![offset_string, bytes, ascii_bytes],
        });
        offset += bytes_per_line;
    }

    let headers = &["Offset", "", "ASCII"];
    Ok(Table::new(TableType::Hex, headers, rows))
}
