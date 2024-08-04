use crate::elf::ElfIdent;
use std::ffi::CStr;
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum BlobError {
    #[error("file not found")]
    FileNotFound,
    #[error("invalid header signature")]
    InvalidHeader,
    #[error("invalid slize")]
    InvalidSliceSize,
}

type Result<T> = std::result::Result<T, BlobError>;

/// Representation of some binary structure
pub struct Blob {
    pub bin_type: BinaryType,
    pub lsb: bool,
    data: Vec<u8>,
}

impl Default for Blob {
    fn default() -> Self {
        Self {
            bin_type: BinaryType::Unknown,
            lsb: false,
            data: Vec::new(),
        }
    }
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut blob = Self {
            bin_type: BinaryType::Unknown,
            lsb: false,
            data,
        };
        blob.guess_file_type()?;
        Ok(blob)
    }

    pub fn get_u8(&self, offset: usize) -> Result<u8> {
        Ok(*self.data.get(offset).ok_or(BlobError::InvalidSliceSize)?)
    }

    pub fn get_u16(&self, offset: usize) -> Result<u16> {
        if self.lsb {
            Ok(u16::from_le_bytes(
                self.data[offset..offset + 2]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            ))
        } else {
            Ok(u16::from_be_bytes(
                self.data[offset..offset + 2]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            ))
        }
    }

    pub fn get_u32(&self, offset: usize) -> Result<u32> {
        if self.lsb {
            Ok(u32::from_le_bytes(
                self.data[offset..offset + 4]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            ))
        } else {
            Ok(u32::from_be_bytes(
                self.data[offset..offset + 4]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            ))
        }
    }

    pub fn get_u64(&self, offset: usize) -> Result<u64> {
        if self.lsb {
            Ok(u64::from_le_bytes(
                self.data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            ))
        } else {
            Ok(u64::from_be_bytes(
                self.data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            ))
        }
    }

    pub fn get_cstr(&self, offset: usize) -> Result<&CStr> {
        CStr::from_bytes_until_nul(&self.data[offset..]).map_err(|_| BlobError::InvalidSliceSize)
    }

    pub fn get_cname(&self, offset: Option<usize>) -> Result<String> {
        match offset {
            Some(name_addr) => {
                let cstr = self.get_cstr(name_addr)?;
                if cstr.is_empty() {
                    Ok("*empty*".to_string())
                } else {
                    Ok(cstr.to_string_lossy().to_string())
                }
            }
            None => Ok("*unnamed*".to_string()),
        }
    }

    fn guess_file_type(&mut self) -> Result<()> {
        if self.data[0..4] == [0x7f, b'E', b'L', b'F'] {
            let elf_ident = ElfIdent::from_slice(
                &self.data[4..16]
                    .try_into()
                    .map_err(|_| BlobError::InvalidSliceSize)?,
            )
            .map_err(|_| BlobError::InvalidHeader)?;
            self.lsb = elf_ident.data != 2;
            self.bin_type = BinaryType::Elf(elf_ident);
            return Ok(());
        }

        if self.data[0..2] == [b'M', b'Z'] {
            self.lsb = true;
            let pe_offset = self.get_u32(0x3c)? as usize;
            if self.data[pe_offset..pe_offset + 4] == [b'P', b'E', 0, 0] {
                self.bin_type = BinaryType::Pe;
                return Ok(());
            }
        }

        self.bin_type = BinaryType::Unknown;

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
