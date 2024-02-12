use std::ffi::CStr;
use std::fmt::{self, Display};
use strum::FromRepr;

use super::Result;
use crate::blob::Blob;

#[repr(u8)]
#[derive(Debug, FromRepr, PartialEq, Eq)]
pub enum SymbolBinding {
    Local = 0,
    Global = 1,
    Weak = 2,
    Unknown = 0xf,
}

#[repr(u8)]
#[derive(Debug, FromRepr, PartialEq, Eq)]
pub enum SymbolType {
    NoType = 0,
    Object = 1,
    Func = 2,
    Section = 3,
    File = 4,
    Common = 5,
    Tls = 6,
    Unknown = 0xf,
}

pub struct Symbol64<'a> {
    // Symbol name, index in string tbl
    name: Option<&'a CStr>,
    symbol_type: SymbolType,
    binding: SymbolBinding,
    // No defined meaning, 0
    other: u8,
    // Associated section index
    index: u16,
    // Value of the symbol
    value: u64,
    // Associated symbol size
    size: u64,
}

impl<'a> Symbol64<'a> {
    pub(super) fn new(blob: &'a Blob, idx: usize, string_table_offset: usize) -> Result<Self> {
        let name_addr = string_table_offset + (blob.get_u32(idx)? as usize);
        let name = if name_addr == 0 {
            None
        } else {
            Some(blob.get_cstring(name_addr)?)
        };
        // Lower 4 bits: symbol type, upper 4 bits: symbol binding
        let info = blob.get_u8(idx + 4)?;

        Ok(Self {
            name,
            symbol_type: SymbolType::from_repr(info & 0xf).unwrap_or(SymbolType::Unknown),
            binding: SymbolBinding::from_repr(info >> 4).unwrap_or(SymbolBinding::Unknown),
            other: blob.get_u8(idx + 5)?,
            index: blob.get_u16(idx + 6)?,
            value: blob.get_u64(idx + 8)?,
            size: blob.get_u64(idx + 16)?,
        })
    }
}

impl<'a> Display for Symbol64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        let name = if let Some(name) = self.name {
            if name.is_empty() {
                "*empty*".to_string()
            } else {
                name.to_string_lossy().to_string()
            }
        } else {
            "*unnamed*".to_string()
        };
        write!(f, " {:7}", format!("{:?}", self.symbol_type))?;
        write!(f, " | {:7}", format!("{:?}", self.binding))?;
        write!(f, " | {:7}", self.other)?;
        write!(f, " | 0x{:016x}", self.value)?;
        write!(f, " | 0x{:016x}", self.size)?;
        write!(f, " | 0x{:04x}", self.index)?;
        write!(f, " | {name:20}")?;
        Ok(())
    }
}
