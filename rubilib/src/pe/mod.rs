use crate::blob::{BinaryType, Blob};
use thiserror::Error;

type Result<T> = std::result::Result<T, PeError>;

#[derive(Error, Debug, Clone)]
pub enum PeError {
    #[error("no pe binary")]
    NoPeBinary,
}

pub struct PeBinary {
    _blob: Blob,
}

impl PeBinary {
    pub fn new(blob: Blob) -> Result<Self> {
        if !matches!(blob.bin_type, BinaryType::Pe) {
            Err(PeError::NoPeBinary)
        } else {
            Ok(Self { _blob: blob })
        }
    }

    pub fn header_info(&self) -> Vec<(String, String)> {
        vec![("Ident".to_string(), "Windows PE binary".to_string())]
    }
}
