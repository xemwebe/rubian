/// Storage class for table data
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum TableType {
    ElfSectionHeader,
    ElfSymbols,
    ElfDynamicSymbols,
}

use TableType::*;

impl Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElfSectionHeader => write!(f, "ELF section headers"),
            ElfSymbols => write!(f, "ELF symbol table"),
            ElfDynamicSymbols => write!(f, "ELF dynamic symbols table"),
        }
    }
}

pub struct Table {
    pub table_type: TableType,
    pub headline: Vec<String>,
    pub rows: Vec<Row>,
}

pub enum RowAction {
    None,
    View,
}

pub struct Row {
    pub action: RowAction,
    pub content: Vec<String>,
}

impl Table {
    pub fn new(table_type: TableType, headers: &[&str], rows: Vec<Row>) -> Self {
        let mut headline = Vec::with_capacity(headers.len());
        for header in headers {
            headline.push(header.to_string());
        }
        Self {
            table_type,
            headline,
            rows,
        }
    }
}
