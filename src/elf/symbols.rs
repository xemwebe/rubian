use std::ffi::CStr;
use std::fmt::{self, Display};
use strum::FromRepr;
use thiserror::Error;

use super::Result;
use crate::blob::{BinaryType, Blob};

pub struct Symbol32 {
    name: u32,
    value: u32,
    size: u32,
    info: u8,
    other: u8,
    sh_idx: u16,
}

pub struct Symbol64<'a> {
    // Symbol name, index in string tbl
    name: Option<&'a CStr>,
    // Type and binding attributes
    info: u8,
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
        Ok(Self {
            name,
            info: blob.get_u8(idx + 4)?,
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
        write!(f, " {:7}", self.info)?;
        write!(f, " | {:7}", self.other)?;
        write!(f, " | 0x{:016x}", self.value)?;
        write!(f, " | 0x{:016x}", self.size)?;
        write!(f, " | 0x{:04x}", self.index)?;
        write!(f, " | {name:20}")?;
        Ok(())
    }
}
