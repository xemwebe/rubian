use std::path::PathBuf;
use anyhow::Result;

use clap::Parser;

mod blob;
mod elf;

#[derive(Parser)]
#[command(author, version, about ,long_about = None)]
struct Cli {
    file: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let file_name = cli.file;
    println!("Loading file {:?}", file_name);

    let mut binary = blob::Blob::from_file(&file_name)?;
    binary.guess_file_type()?;
    match binary.bin_type {
        blob::BinaryType::Elf(_) => {
            let mut elf_binary = elf::ElfBinary::new(&binary)?;
            println!("{}", elf_binary.header_info()?);
            println!("\nSection header table:");
            println!("{}", elf_binary.section_headers_info()?);
            println!("\nSymbol table:");
            println!("{}", elf_binary.symbols_info()?);
        }
        _ => { println!("Not supported yet"); }
    }

    Ok(())
}

