#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use anyhow::Result;
use clap::Parser;
use log::{debug, info};
use std::path::PathBuf;
use tauri::{CustomMenuItem, Manager, Menu, Submenu, Window};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    tauri::Builder::default()
        .menu(
            Menu::new().add_submenu(Submenu::new(
                "File",
                Menu::new()
                    .add_item(CustomMenuItem::new("close", "Close").accelerator("cmdOrControl+Q"))
                    .add_item(
                        CustomMenuItem::new("open", "Open File").accelerator("cmdOrControl+O"),
                    ),
            )),
        )
        .on_menu_event(|event| match event.menu_item_id() {
            "close" => {
                event.window().close().unwrap();
            }
            "open" => {}
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
