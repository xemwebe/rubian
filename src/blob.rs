use crate::elf::ElfIdent;
use std::ffi::CStr;
use std::fmt::{self, Display};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("invalid binary")]
    InvalidSliceSize,
    #[error("file not found")]
    FileNotFound,
    #[error("invalid header signature")]
    InvalidHeader,
}

type Result<T> = std::result::Result<T, BinaryError>;

/// Representation of some binary structure
pub struct Blob {
    pub bin_type: BinaryType,
    pub lsb: bool,
    data: Vec<u8>,
}

impl Blob {
    pub fn from_file(file_name: &Path) -> Result<Self> {
        let file = fs::read(file_name).map_err(|_| BinaryError::FileNotFound)?;
        Ok(Self {
            bin_type: BinaryType::Unknown,
            lsb: true,
            data: file,
        })
    }

    pub fn get_u8(&self, offset: usize) -> Result<u8> {
        Ok(*self.data.get(offset).ok_or(BinaryError::InvalidSliceSize)?)
    }

    pub fn get_u16(&self, offset: usize) -> Result<u16> {
        if self.lsb {
            Ok(u16::from_le_bytes(
                self.data[offset..offset + 2]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            ))
        } else {
            Ok(u16::from_be_bytes(
                self.data[offset..offset + 2]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            ))
        }
    }

    pub fn get_u32(&self, offset: usize) -> Result<u32> {
        if self.lsb {
            Ok(u32::from_le_bytes(
                self.data[offset..offset + 4]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            ))
        } else {
            Ok(u32::from_be_bytes(
                self.data[offset..offset + 4]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            ))
        }
    }

    pub fn get_u64(&self, offset: usize) -> Result<u64> {
        if self.lsb {
            Ok(u64::from_le_bytes(
                self.data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            ))
        } else {
            Ok(u64::from_be_bytes(
                self.data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            ))
        }
    }

    pub fn get_cstring(&self, offset: usize) -> Result<&CStr> {
        CStr::from_bytes_until_nul(&self.data[offset..]).map_err(|_| BinaryError::InvalidSliceSize)
    }

    pub fn guess_file_type(&mut self) -> Result<()> {
        if self.data[0..4] == [0x7f, b'E', b'L', b'F'] {
            let elf_ident = ElfIdent::from_slice(
                &self.data[4..16]
                    .try_into()
                    .map_err(|_| BinaryError::InvalidSliceSize)?,
            )
            .map_err(|_| BinaryError::InvalidHeader)?;
            self.lsb = elf_ident.data != 2;
            self.bin_type = BinaryType::Elf(elf_ident);
            return Ok(());
        }

        if self.data[0..4] == [b'M', b'Z', 0, 0] {
            self.bin_type = BinaryType::Pe;
            return Ok(());
        }

        Ok(())
    }
}

pub enum BinaryType {
    Elf(ElfIdent),
    Pe,
    Unknown,
}

impl Display for BinaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            BinaryType::Elf(elf_ident) => {
                write!(f, "elf")?;
                write!(f, "{elf_ident}")
            }
            BinaryType::Pe => write!(f, "pe"),
            BinaryType::Unknown => write!(f, "unknown"),
        }
    }
}
