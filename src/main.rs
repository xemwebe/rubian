#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use anyhow::Result;
use clap::Parser;
use log::{debug, info};
use std::path::PathBuf;
mod blob;
mod elf;

#[derive(Parser)]
#[command(author, version, about ,long_about = None)]
struct Cli {
    file: Option<PathBuf>,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let cli = Cli::parse();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Rubian - a binary analysis tool in rust",
        options,
        Box::new(move |_cc| Box::<RubianApp>::new(RubianApp::new(cli))),
    )
}

trait FileInfo {
    fn info(&mut self, ui: &mut egui::Ui);
}

struct NoFile {
    message: String,
}

impl NoFile {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl FileInfo for NoFile {
    fn info(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.message);
    }
}

struct RubianApp {
    file: Option<std::path::PathBuf>,
    file_info: Box<dyn FileInfo>,
}

impl Default for RubianApp {
    fn default() -> Self {
        Self {
            file: None,
            file_info: Box::new(NoFile {
                message: "no file loaded".to_string(),
            }),
        }
    }
}

impl RubianApp {
    fn new(cli: Cli) -> Self {
        Self {
            file: cli.file,
            file_info: Box::new(NoFile::new("no file loaded")),
        }
    }

    fn load_file(&mut self, path: std::path::PathBuf) {
        info!("Loading new file {:?}", path);
        if let Ok(mut binary) = blob::Blob::from_file(&path) {
            if binary.guess_file_type().is_err() {
                self.file_info = Box::new(NoFile::new(&format!(
                    "File type of file {} is not supported",
                    path.to_string_lossy()
                )));
                return;
            }
            self.file_info = match binary.bin_type {
                blob::BinaryType::Elf(_) => {
                    if let Ok(elf) = ElfBinaryInfo::new(binary) {
                        Box::new(elf)
                    } else {
                        Box::new(NoFile::new("Header of ELF binary is corrupt"))
                    }
                }
                blob::BinaryType::Pe => Box::new(NoFile::new("File is Windows binary")),
                blob::BinaryType::Unknown => Box::new(NoFile::new("unknown file type")),
            };
            self.file = Some(path);
        } else {
            self.file_info = Box::new(NoFile::new(&format!(
                "Failed to load file {}",
                path.to_string_lossy()
            )));
        }
    }
}

struct ElfBinaryInfo {
    elf: elf::ElfBinary,
}

impl ElfBinaryInfo {
    fn new(binary: blob::Blob) -> Result<Self> {
        Ok(Self {
            elf: elf::ElfBinary::new(binary)?,
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
    }
}
// fn file_info_output(&mut self, ui: &mut egui::Ui) -> Result<()> {
//     match &self.file {
//         Some(path) => {
//             if self.data.is_none() || self.reload {
//                 self.reload = false;
//                 info!("Loading {}", path.to_string_lossy());
//                 self.data = Some(blob::Blob::from_file(path)?);
//                 debug!("Loading was successfull");
//             }
//             if let Some(binary) = &mut self.data {
//                 ui.add(egui::Label::new(&format!(
//                     "Anylizing file {}",
//                     path.to_string_lossy()
//                 )));
//                 binary.guess_file_type()?;
//                 debug!("file type guessed");
//                 match binary.bin_type {
//                     blob::BinaryType::Elf(_) => {
//                         let mut elf_binary = elf::ElfBinary::new(&binary)?;
//                         debug!("Elf binary loaded");
//                         let mut output = format!("{}", elf_binary.header_info()?);
//                         debug!("header info created");
//                         output = format!("{output}\nSection header table:");
//                         output = format!("{output}{}", elf_binary.section_headers_info()?);
//                         debug!("section info created");
//                         output = format!("{output}\nSymbol table:");
//                         output = format!("{output}{}", elf_binary.symbols_info()?);
//                         debug!("symbol info created");
//                         output = format!("{output}\nDynamic symbol table:");
//                         output = format!("{output}{}", elf_binary.dyn_symbols_info()?);
//                         debug!("dyn symbols created");
//                         ui.add(egui::Label::new(&output));
//                     }
//                     _ => {
//                         ui.add(egui::Label::new("File type is not supported"));
//                     }
//                 }
//             } else {
//                 ui.add(egui::Label::new("Failed to load file"));
//             }
//         }
//         None => {
//             ui.add(egui::Label::new("Open a file to analyze"));
//         }
//     }

//     Ok(())
// }

impl eframe::App for RubianApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Open file..").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.load_file(path);
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.file_info.info(ui);
            });
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    self.load_file(path.clone());
                }
            }
        });
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}
