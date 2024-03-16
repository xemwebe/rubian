use crate::file_info::FileInfo;
use anyhow::Result;
use eframe::egui;
use rubian_core::{blob, elf};

pub struct ElfBinaryInfo {
    elf: elf::ElfBinary,
    content: Vec<String>,
}

impl ElfBinaryInfo {
    pub fn new(binary: blob::Blob) -> Result<Self> {
        Ok(Self {
            elf: elf::ElfBinary::new(binary)?,
            content: Vec::new(),
        })
    }

    fn elf_header(&self) -> String {
        match self.elf.header_info() {
            Ok(info) => info,
            Err(err) => format!("Error: {}", err),
        }
    }
}

impl FileInfo for ElfBinaryInfo {
    fn info(&mut self, ui: &mut egui::Ui) {
        ui.label(self.elf_header());

        if ui.button("View section header").clicked() {
            match self.elf.section_headers_info() {
                Ok(s) => {
                    self.content.push(s);
                }
                Err(e) => {
                    self.content
                        .push(format!("Failed to read section header: {}", e));
                }
            }
        }

        if ui.button("View symbols").clicked() {
            match self.elf.symbols_info() {
                Ok(s) => {
                    self.content.push(s);
                }
                Err(e) => {
                    self.content.push(format!("Failed symbol table: {}", e));
                }
            }
        }

        if ui.button("View dynamic symbols").clicked() {
            match self.elf.dyn_symbols_info() {
                Ok(s) => {
                    self.content.push(s);
                }
                Err(e) => {
                    self.content
                        .push(format!("Failed to read dynamic symbols table: {}", e));
                }
            }
        }

        for s in &self.content {
            ui.label(s);
        }
    }
}
