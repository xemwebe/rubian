use crate::file_info::FileInfo;
use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rubian_core::{blob, elf, table};

struct TableInfo {
    table: table::Table,
}

impl TableInfo {
    fn new(table: table::Table) -> Self {
        Self { table }
    }

    fn draw(&self, ui: &mut egui::Ui) {
        ui.heading(self.table.table_type.to_string());
        ui.add_space(15.0);
        TableBuilder::new(ui)
            .striped(true)
            .columns(Column::auto().resizable(true), self.table.headline.len())
            .header(20.0, |mut header| {
                for h in &self.table.headline {
                    header.col(|ui| {
                        ui.heading(h);
                    });
                }
            })
            .body(|mut body| {
                for table_row in &self.table.rows {
                    body.row(15.0, |mut row| {
                        for table_col in table_row {
                            row.col(|ui| {
                                ui.label(table_col);
                            });
                        }
                    });
                }
            });
        ui.add_space(30.0);
    }
}

pub struct ElfBinaryInfo {
    elf: elf::ElfBinary,
    content: Vec<String>,
    tables: Vec<TableInfo>,
}

impl ElfBinaryInfo {
    pub fn new(binary: blob::Blob) -> Result<Self> {
        Ok(Self {
            elf: elf::ElfBinary::new(binary)?,
            content: Vec::new(),
            tables: Vec::new(),
        })
    }

    fn elf_header(&self) -> String {
        match self.elf.header_info() {
            Ok(info) => info,
            Err(err) => format!("Error: {}", err),
        }
    }

    fn draw_tables(&self, ui: &mut egui::Ui) {
        for (idx, table) in self.tables.iter().enumerate() {
            ui.push_id(idx, |ui| table.draw(ui));
        }
    }
}

impl FileInfo for ElfBinaryInfo {
    fn info(&mut self, ui: &mut egui::Ui) {
        ui.label(self.elf_header());

        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            if ui.button("View section header").clicked() {
                match self.elf.section_headers_table() {
                    Ok(t) => {
                        self.tables.push(TableInfo::new(t));
                    }
                    Err(e) => {
                        self.content
                            .push(format!("Failed to read seciont header: {}", e));
                    }
                }
            }

            if ui.button("View symbols").clicked() {
                match self.elf.symbols_table() {
                    Ok(t) => {
                        self.tables.push(TableInfo::new(t));
                    }
                    Err(e) => {
                        self.content.push(format!("Failed symbol table: {}", e));
                    }
                }
            }

            if ui.button("View dynamic symbols").clicked() {
                match self.elf.dyn_symbols_table() {
                    Ok(t) => {
                        self.tables.push(TableInfo::new(t));
                    }
                    Err(e) => {
                        self.content
                            .push(format!("Failed to read dynamic symbols table: {}", e));
                    }
                }
            }
        });
        ui.add_space(20.0);

        self.draw_tables(ui);

        for s in &self.content {
            ui.label(s);
            ui.add_space(20.0);
        }
    }
}
