use crate::blob::{BinaryType, Blob, BlobError};
use crate::elf;
use crate::pe;
use std::{
    fmt::{self, Display},
    fs,
    path::Path,
};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum BinaryError {
    #[error("corrupt pe binary")]
    NoPeBinary(#[from] pe::PeError),
    #[error("corrupt elf binary")]
    NoElfBinary(#[from] elf::ElfError),
    #[error("corrupt binary blob")]
    BlobCorrupted(#[from] BlobError),
}

type Result<T> = std::result::Result<T, BinaryError>;

pub enum Binary {
    Elf(elf::ElfBinary),
    Pe,
    Unknown,
}

impl Binary {
    pub fn from_file(file_name: &Path) -> Result<Self> {
        let data = fs::read(file_name).map_err(|_| BlobError::FileNotFound)?;
        let blob = Blob::new(data)?;
        Self::new(blob)
    }

    pub fn new(blob: Blob) -> Result<Self> {
        match blob.bin_type {
            BinaryType::Elf(_) => {
                let elf_binary = elf::ElfBinary::new(blob)?;
                Ok(Self::Elf(elf_binary))
            }
            BinaryType::Pe => Ok(Self::Pe),
            _ => Ok(Self::Unknown),
        }
    }

    pub fn update(&mut self, blob: Blob) -> Result<()> {
        *self = Self::new(blob)?;
        Ok(())
    }

    pub fn file_info(&self) -> Vec<(String, String)> {
        match self {
            Binary::Elf(elf_binary) => elf_binary.header_info(),
            Binary::Pe => {
                vec![("Ident".to_string(), "Windows PE binary".to_string())]
            },
            Binary::Unknown => {
                vec![("Ident".to_string(), "Unknown binary".to_string())]
            }
        }
    }
}

impl Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            Binary::Elf(elf_binary) => {
                write!(f, "elf")?;
                write!(f, "{}", elf_binary.ident())
            }
            Binary::Pe => write!(f, "pe"),
            Binary::Unknown => write!(f, "unknown"),
        }
    }
}

impl Default for Binary {
    fn default() -> Self {
        Self::Unknown
    }
}
