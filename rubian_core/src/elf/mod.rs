use crate::blob::{BinaryError, BinaryType, Blob};
use crate::table::{Row, RowAction, Table, TableType};
use std::fmt::{self, Display};
use strum::FromRepr;
use thiserror::Error;

type Result<T> = std::result::Result<T, ElfError>;

mod symbols;

use symbols::Symbol64;

#[derive(Error, Debug)]
pub enum ElfError {
    #[error("no elf binary")]
    NoElfBinary,
    #[error("invalid binary")]
    InvalidBinary(#[from] BinaryError),
    #[error("internal error")]
    InternalError,
}

#[derive(Clone, Debug)]
pub struct ElfIdent {
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
}

impl ElfIdent {
    pub fn from_slice(slice: &[u8; 12]) -> Result<Self> {
        Ok(Self {
            class: slice[0],
            data: slice[1],
            version: slice[2],
            os_abi: slice[3],
            abi_version: slice[4],
            padding: slice[5..12].try_into().unwrap(),
        })
    }
}

impl Display for ElfIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self.class {
            1 => write!(f, "32"),
            2 => write!(f, "64"),
            _ => write!(f, "??"),
        }?;
        match self.data {
            1 => write!(f, " little endian"),
            2 => write!(f, " big endian"),
            _ => write!(f, " ???"),
        }?;
        match self.os_abi {
            0 => write!(f, " UNIX System V"),
            _ => write!(f, " custom ABI"),
        }
    }
}

pub struct ElfHeader {
    elf_type: u16,
    machine: MachineType,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(u16)]
#[derive(Debug, FromRepr, PartialEq, Eq)]
pub enum MachineType {
    Unspecific = 0x00,
    AttWe32100 = 0x01,
    Sparc = 0x02,
    X86 = 0x03,
    Motorola68000 = 0x04,
    Motorola88000 = 0x05,
    IntelMcu = 0x06,
    Intel80860 = 0x07,
    Mips = 0x08,
    IbmSystem370 = 0x09,
    MipsRs3000LittleEndian = 0x0a,
    HewlettPackardPaRisc = 0x0f,
    Intel80960 = 0x13,
    PowerPc = 0x14,
    PowerPC64 = 0x15,
    S390 = 0x16,
    IbmSpuSpc = 0x17,
    NecV800 = 0x24,
    FujitsuFr20 = 0x25,
    TrwRh32 = 0x26,
    MotorolaRce = 0x27,
    Arm = 0x28,
    DigitalAlpha = 0x29,
    SuperH = 0x2a,
    SparcV9 = 0x2b,
    SiemensTriCore = 0x2c,
    ArgonautRiscCore = 0x2d,
    HitachiH8300 = 0x2E,
    HitachiH8300H = 0x2F,
    HitachiH8S = 0x30,
    HitachiH8500 = 0x31,
    Ia64 = 0x32,
    StanfordMipsX = 0x33,
    MotorolaColdFire = 0x34,
    MotorolaM68Hc12 = 0x35,
    FujitsuMma = 0x36,
    SiemensPcp = 0x37,
    SonyNCpu = 0x38,
    DensoNdr1 = 0x39,
    MotorolaStarCore = 0x3A,
    ToyotaMe16 = 0x3B,
    STMicroelectronicsSt100 = 0x3C,
    AdvancedLogicCorpTinyJ = 0x3D,
    AmdX64 = 0x3E,
    SonyDsp = 0x3F,
    DecPdp10 = 0x40,
    DecPdp11 = 0x41,
    SiemensFx66 = 0x42,
    StMicroelectronicsSt9p = 0x43,
    StMicroelectronicsSt7 = 0x44,
    MotorolaMc68Hc16 = 0x45,
    MotorolaMc68Hc11 = 0x46,
    MotorolaMc68Hc08 = 0x47,
    MotorolaMc68Hc05 = 0x48,
    SiliconGraphicsSvx = 0x49,
    StMicroelectronicsSt19 = 0x4A,
    DigitalVax = 0x4B,
    AxisCommunications = 0x4C,
    InfineonTechnologies = 0x4D,
    Element14 = 0x4E,
    LsiLogic = 0x4F,
    Tms320C6000 = 0x8C,
    McstElbrus = 0xAF,
    Arm64 = 0xB7,
    ZilogZ80 = 0xDC,
    RiscV = 0xF3,
    BerkeleyPacketFilter = 0xF7,
    Wdc65C816 = 0x101,
    Reserved = 0xffff,
}

impl Display for MachineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            MachineType::Unspecific => write!(f, "No specific instruction set"),
            MachineType::AttWe32100 => write!(f, "AT&T WE 32100"),
            MachineType::Sparc => write!(f, "Sparc"),
            MachineType::X86 => write!(f, "x86"),
            MachineType::Motorola68000 => write!(f, "Motorola 68000 (M68k)"),
            MachineType::Motorola88000 => write!(f, "Motorola 88000 (M88k)"),
            MachineType::IntelMcu => write!(f, "Intel MCU"),
            MachineType::Intel80860 => write!(f, "Intel 80860"),
            MachineType::Mips => write!(f, "MIPS"),
            MachineType::IbmSystem370 => write!(f, "IBM System/370"),
            MachineType::MipsRs3000LittleEndian => write!(f, "MIPS RS3000 Little-endian"),
            MachineType::HewlettPackardPaRisc => write!(f, "Hewlett-Packard PA-RISC"),
            MachineType::Intel80960 => write!(f, "Intel 80960"),
            MachineType::PowerPc => write!(f, "PowerPC"),
            MachineType::PowerPC64 => write!(f, "PowerPC (64-bit)"),
            MachineType::S390 => write!(f, "S390, including S390x"),
            MachineType::IbmSpuSpc => write!(f, "IBM SPU/SPC"),
            MachineType::NecV800 => write!(f, "NEC V800"),
            MachineType::FujitsuFr20 => write!(f, "Fujitsu FR20"),
            MachineType::TrwRh32 => write!(f, "TRW RH-32"),
            MachineType::MotorolaRce => write!(f, "Motorola RCE"),
            MachineType::Arm => write!(f, "Arm (up to Armv7/AArch32)"),
            MachineType::DigitalAlpha => write!(f, "Digital Alpha"),
            MachineType::SuperH => write!(f, "SuperH"),
            MachineType::SparcV9 => write!(f, "SPARC Version 9"),
            MachineType::SiemensTriCore => write!(f, "Siemens TriCore embedded processor"),
            MachineType::ArgonautRiscCore => write!(f, "Argonaut RISC Core"),
            MachineType::HitachiH8300 => write!(f, "Hitachi H8/300"),
            MachineType::HitachiH8300H => write!(f, "Hitachi H8/300H"),
            MachineType::HitachiH8S => write!(f, "Hitachi H8S"),
            MachineType::Ia64 => write!(f, "IA-64"),
            MachineType::HitachiH8500 => write!(f, "Hitachi H8/500"),
            MachineType::StanfordMipsX => write!(f, "Stanford MIPS-X"),
            MachineType::MotorolaColdFire => write!(f, "Motorola ColdFire"),
            MachineType::MotorolaM68Hc12 => write!(f, "Motorola M68HC12"),
            MachineType::FujitsuMma => write!(f, "Fujitsu MMA Multimedia Accelerator"),
            MachineType::SiemensPcp => write!(f, "Siemens PCP"),
            MachineType::SonyNCpu => write!(f, "Sony nCPU embedded RISC processor"),
            MachineType::DensoNdr1 => write!(f, "Denso NDR1 microprocessor"),
            MachineType::MotorolaStarCore => write!(f, "Motorola Star*Core processor"),
            MachineType::ToyotaMe16 => write!(f, "Toyota ME16 processor"),
            MachineType::STMicroelectronicsSt100 => write!(f, "STMicroelectronics ST100 processor"),
            MachineType::AdvancedLogicCorpTinyJ => {
                write!(f, "Advanced Logic Corp. TinyJ embedded processor family")
            }
            MachineType::AmdX64 => write!(f, "AMD x86-64"),
            MachineType::SonyDsp => write!(f, "Sony DSP Processor"),
            MachineType::DecPdp10 => write!(f, "Digital Equipment Corp. PDP-10"),
            MachineType::DecPdp11 => write!(f, "Digital Equipment Corp. PDP-11"),
            MachineType::SiemensFx66 => write!(f, "Siemens FX66 microcontroller"),
            MachineType::StMicroelectronicsSt9p => {
                write!(f, "STMicroelectronics ST9+ 8/16 bit microcontroller")
            }
            MachineType::StMicroelectronicsSt7 => {
                write!(f, "STMicroelectronics ST7 8-bit microcontroller")
            }
            MachineType::MotorolaMc68Hc16 => write!(f, "Motorola MC68HC16 Microcontroller"),
            MachineType::MotorolaMc68Hc11 => write!(f, "Motorola MC68HC11 Microcontroller"),
            MachineType::MotorolaMc68Hc08 => write!(f, "Motorola MC68HC08 Microcontroller"),
            MachineType::MotorolaMc68Hc05 => write!(f, "Motorola MC68HC05 Microcontroller"),
            MachineType::SiliconGraphicsSvx => write!(f, "Silicon Graphics SVx"),
            MachineType::StMicroelectronicsSt19 => {
                write!(f, "STMicroelectronics ST19 8-bit microcontroller")
            }
            MachineType::DigitalVax => write!(f, "Digital VAX"),
            MachineType::AxisCommunications => {
                write!(f, "Axis Communications 32-bit embedded processor")
            }
            MachineType::InfineonTechnologies => {
                write!(f, "Infineon Technologies 32-bit embedded processor")
            }
            MachineType::Element14 => write!(f, "Element 14 64-bit DSP Processor"),
            MachineType::LsiLogic => write!(f, "LSI Logic 16-bit DSP Processor"),
            MachineType::Tms320C6000 => write!(f, "TMS320C6000 Family"),
            MachineType::McstElbrus => write!(f, "MCST Elbrus e2k"),
            MachineType::Arm64 => write!(f, "Arm 64-bits (Armv8/AArch64)"),
            MachineType::ZilogZ80 => write!(f, "Zilog Z80"),
            MachineType::RiscV => write!(f, "RISC-V"),
            MachineType::BerkeleyPacketFilter => write!(f, "Berkeley Packet Filter"),
            MachineType::Wdc65C816 => write!(f, "WDC 65C816"),
            MachineType::Reserved => write!(f, "reserved"),
        }
    }
}

pub struct SectionHeader {
    // Section name (string tbl index)
    name: Option<usize>,
    // Section type
    section_type: ElfSectionType,
    // Section flags
    flags: u64,
    // Section virtual addr at execution
    addr: u64,
    // Section file offset
    offset: u64,
    // Section size in bytes
    size: u64,
    // Link to another section
    link: u32,
    // Additional section information
    info: u32,
    // Section alignment
    addr_align: u64,
    // Entry size if section holds table
    ent_size: u64,
}

impl SectionHeader {
    fn new(blob: &Blob, offset: usize, header_string_table_offset: usize) -> Result<Self> {
        let name_addr = header_string_table_offset + (blob.get_u32(offset)? as usize);
        let name = if name_addr == 0 {
            None
        } else {
            Some(name_addr)
        };
        let section_type =
            ElfSectionType::from_repr(blob.get_u32(offset + 4)?).unwrap_or(ElfSectionType::Unknown);
        if section_type == ElfSectionType::Unknown {
            eprintln!(
                "Unknown section type {} (0x{:016x}) found",
                blob.get_u32(offset + 4)?,
                blob.get_u32(offset + 4)?
            );
        }
        Ok(Self {
            name,
            section_type,
            flags: blob.get_u64(offset + 8)?,
            addr: blob.get_u64(offset + 16)?,
            offset: blob.get_u64(offset + 24)?,
            size: blob.get_u64(offset + 32)?,
            link: blob.get_u32(offset + 40)?,
            info: blob.get_u32(offset + 44)?,
            addr_align: blob.get_u64(offset + 48)?,
            ent_size: blob.get_u64(offset + 56)?,
        })
    }

    fn flags_as_string(&self) -> String {
        let mut flag_string = String::new();
        if self.flags & 0x1 != 0 {
            flag_string = format!("{flag_string}w");
        }
        if self.flags & 0x2 != 0 {
            flag_string = format!("{flag_string}a");
        }
        if self.flags & 0x4 != 0 {
            flag_string = format!("{flag_string}x");
        }
        if self.flags & 0x100000 != 0 {
            flag_string = format!("{flag_string}l");
        }
        if self.flags & 0x200000 != 0 {
            flag_string = format!("{flag_string}i");
        }
        if self.flags & 0xf0000000 != 0 {
            flag_string = format!("{flag_string}m");
        }
        flag_string
    }

    fn to_vec(&self, blob: &Blob) -> Result<Vec<String>> {
        let mut v = Vec::with_capacity(10);
        v.push(blob.get_cname(self.name)?);
        v.push(format!("{:?}", self.section_type));
        v.push(self.flags_as_string());
        v.push(format!("0x{:016x}", self.addr));
        v.push(format!("0x{:016x}", self.offset));
        v.push(format!("0x{:016x}", self.size));
        v.push(self.link.to_string());
        v.push(self.info.to_string());
        v.push(self.addr_align.to_string());
        v.push(self.ent_size.to_string());
        Ok(v)
    }
}

impl ElfHeader {
    pub fn new(blob: &Blob) -> Result<Self> {
        let machine = if let Some(machine) = MachineType::from_repr(blob.get_u16(18)?) {
            machine
        } else {
            MachineType::Reserved
        };
        Ok(Self {
            elf_type: blob.get_u16(16)?,
            machine,
            version: blob.get_u32(20)?,
            entry: blob.get_u64(24)?,
            phoff: blob.get_u64(32)?,
            shoff: blob.get_u64(40)?,
            flags: blob.get_u32(48)?,
            ehsize: blob.get_u16(52)?,
            phentsize: blob.get_u16(54)?,
            phnum: blob.get_u16(56)?,
            shentsize: blob.get_u16(58)?,
            shnum: blob.get_u16(60)?,
            shstrndx: blob.get_u16(62)?,
        })
    }

    pub fn info(&self, full: bool) -> String {
        let mut s = format!(
            "type: {}",
            match self.elf_type {
                0 => "none",
                1 => "relocatable",
                2 => "executable",
                3 => "shared object (pie)",
                4 => "core file",
                0xfe00 | 0xfeff => "OS specific",
                0xff00 | 0xffff => "processor-specific",
                _ => "unknown",
            }
        );
        s = format!("{s}, entry point: 0x{:016x}", self.entry);
        if full {
            s = format!("{s}\nMachine type: {}", self.machine);
            s = format!(
                "{s}\nProgram header offset: 0x{:016x}, size: 0x{:04x}, count: {:6}",
                self.phoff, self.phentsize, self.phnum
            );
            s = format!(
                "{s}\nSection header offset: 0x{:016x}, size: 0x{:04x}, count: {:6}",
                self.shoff, self.shentsize, self.shnum
            );
            s = format!("{s}\nExecutable header size: 0x{:04x}", self.ehsize);
            s = format!("{s}\nSection header string offset: 0x{:04x}", self.shstrndx);
            s = format!("{s}\nVersion: {}", self.version);
            s = format!("{s}\nFlags: 0x{:016x}", self.flags);
        }
        s
    }
}

pub struct ElfBinary {
    blob: Blob,
    id: ElfIdent,
    header: ElfHeader,
    section_headers: Vec<SectionHeader>,
    symbols: Vec<Symbol64>,
    dyn_symbols: Vec<Symbol64>,
    header_string_table_offset: usize,
}

impl ElfBinary {
    pub fn new(blob: Blob) -> Result<Self> {
        let id = match &blob.bin_type {
            BinaryType::Elf(elf_ident) => Ok(elf_ident),
            _ => Err(ElfError::NoElfBinary),
        }?;
        let id = (*id).clone();

        let header = ElfHeader::new(&blob)?;
        let string_table_header_offset =
            (header.shoff + (header.shentsize as u64) * (header.shstrndx as u64)) as usize;
        let header_string_table_offset = blob.get_u64(string_table_header_offset + 24)? as usize;
        Ok(Self {
            blob,
            id,
            header,
            section_headers: Vec::new(),
            symbols: Vec::new(),
            dyn_symbols: Vec::new(),
            header_string_table_offset,
        })
    }

    pub fn header_info(&self) -> Result<String> {
        Ok(format!("{}\n{}", self.id, self.header.info(true)))
    }

    pub fn section_headers_table(&mut self) -> Result<Table> {
        self.get_sections()?;
        let headers = [
            "Nr.",
            "Name",
            "Type",
            "Flags",
            "Address",
            "FileOffset",
            "Size",
            "Link",
            "Info",
            "Address Alignment",
            "Entries Size",
        ];
        let mut rows = Vec::with_capacity(self.section_headers.len());
        for (idx, sec) in self.section_headers.iter().enumerate() {
            let mut v = sec.to_vec(&self.blob)?;
            let mut content = Vec::with_capacity(v.len() + 1);
            content.push(idx.to_string());
            content.append(&mut v);
            if headers.len() != content.len() {
                return Err(ElfError::InternalError);
            }
            rows.push(Row {
                content,
                action: RowAction::None,
            });
        }
        Ok(Table::new(TableType::ElfSectionHeader, &headers, rows))
    }

    const SYMBOL_HEADERS: [&'static str; 8] = [
        "Nr.", "Type", "Binding", "Other", "Value", "Size", "SecIdx", "Name",
    ];

    pub fn symbols_table(&mut self) -> Result<Table> {
        self.get_symbols()?;
        let mut rows = Vec::with_capacity(self.symbols.len());
        for (idx, symbol) in self.symbols.iter().enumerate() {
            let mut v = symbol.to_vec(&self.blob)?;
            let mut content = Vec::with_capacity(v.len() + 1);
            content.push(idx.to_string());
            content.append(&mut v);
            if Self::SYMBOL_HEADERS.len() != content.len() {
                return Err(ElfError::InternalError);
            }
            rows.push(Row {
                content,
                action: RowAction::None,
            });
        }
        Ok(Table::new(
            TableType::ElfSymbols,
            &Self::SYMBOL_HEADERS,
            rows,
        ))
    }

    pub fn dyn_symbols_table(&mut self) -> Result<Table> {
        self.get_dyn_symbols()?;
        let mut rows = Vec::with_capacity(self.dyn_symbols.len());
        for (idx, symbol) in self.dyn_symbols.iter().enumerate() {
            let mut v = symbol.to_vec(&self.blob)?;
            let mut content = Vec::with_capacity(v.len() + 1);
            content.push(idx.to_string());
            content.append(&mut v);
            if Self::SYMBOL_HEADERS.len() != content.len() {
                return Err(ElfError::InternalError);
            }
            rows.push(Row {
                content,
                action: RowAction::None,
            });
        }
        Ok(Table::new(
            TableType::ElfDynamicSymbols,
            &Self::SYMBOL_HEADERS,
            rows,
        ))
    }

    fn get_sections(&mut self) -> Result<()> {
        if self.section_headers.is_empty() {
            let mut idx = self.header.shoff as usize;
            for _ in 0..self.header.shnum {
                self.section_headers.push(SectionHeader::new(
                    &self.blob,
                    idx,
                    self.header_string_table_offset,
                )?);
                idx += self.header.shentsize as usize;
            }
        }
        Ok(())
    }

    fn get_section_offset(&mut self, section_name: &str) -> Result<Option<usize>> {
        self.get_sections()?;
        for section in &self.section_headers {
            if let Some(name) = section.name {
                let name = self.blob.get_cstr(name)?;
                if name.to_bytes() == section_name.as_bytes() {
                    return Ok(Some(section.offset as usize));
                }
            }
        }
        Ok(None)
    }

    fn get_symbols(&mut self) -> Result<()> {
        self.get_sections()?;
        if self.symbols.is_empty() {
            if let Some(string_table_offset) = self.get_section_offset(".strtab")? {
                for section in &self.section_headers {
                    if section.section_type == ElfSectionType::SymTab {
                        let mut idx = section.offset as usize;
                        let end = idx + section.size as usize;
                        while idx < end {
                            self.symbols
                                .push(Symbol64::new(&self.blob, idx, string_table_offset)?);
                            idx += section.ent_size as usize;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_dyn_symbols(&mut self) -> Result<()> {
        self.get_sections()?;
        if self.dyn_symbols.is_empty() {
            if let Some(string_table_offset) = self.get_section_offset(".dynstr")? {
                for section in &self.section_headers {
                    if section.section_type == ElfSectionType::DynSym {
                        let mut idx = section.offset as usize;
                        let end = idx + section.size as usize;
                        while idx < end {
                            self.dyn_symbols.push(Symbol64::new(
                                &self.blob,
                                idx,
                                string_table_offset,
                            )?);
                            idx += section.ent_size as usize;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[repr(u32)]
#[derive(Debug, FromRepr, PartialEq, Eq)]
pub enum ElfSectionType {
    Null = 0x0,          // No associated section (inactive entry).
    ProgBits = 0x1,      // Program-defined contents.
    SymTab = 0x2,        // Symbol table.
    StrTab = 0x3,        // String table.
    Rela = 0x4,          // Relocation entries; explicit addends.
    Hash = 0x5,          // Symbol hash table.
    Dynamic = 0x6,       // Information for dynamic linking.
    Note = 0x7,          // Information about the file.
    NoBits = 0x8,        // Data occupies no space in the file.
    Rel = 0x9,           // Relocation entries; no explicit addends.
    SHLib = 0xa,         // Reserved.
    DynSym = 0xb,        // Symbol table.
    InitArray = 0xe,     // Pointers to initialization functions.
    FiniArray = 0xf,     // Pointers to termination functions.
    PreInitArray = 0x10, // Pointers to pre-init functions.
    Group = 0x11,        // Section group.
    SymTabShNdx = 0x12,  // Indices for SHN_XINDEX entries.
    Relr = 0x13,         // Relocation entries; only offsets.

    // Start of system-specific types
    AndroidRel = 0x60000001,
    AndroidRela = 0x60000002,
    LlvmOdrTab = 0x6fff4c00,             // LLVM ODR table.
    LlvmLinkerOptions = 0x6fff4c01,      // LLVM Linker Options.
    LlvmAddrSig = 0x6fff4c03,            // List of address-significant symbols for safe ICF.
    LlvmDependentLibraries = 0x6fff4c04, // LLVM Dependent Library Specifiers.
    LlvmSymPart = 0x6fff4c05,            // Symbol partition specification.
    LlvmPartEhdr = 0x6fff4c06,           // ELF header for loadable partition.
    LlvmPartPhDR = 0x6fff4c07,           // Phdrs for loadable partition.
    LlvmBbAddrMapV0 = 0x6fff4c08, // LLVM Basic Block Address Map (old version kept for backward-compatibility).
    LlvmCallGraphProfile = 0x6fff4c09, // LLVM Call Graph Profile.
    LlvmBbAddrMap = 0x6fff4c0a,   // LLVM Basic Block Address Map.
    LlvmOffloading = 0x6fff4c0b,  // LLVM device offloading data.
    LlvmLto = 0x6fff4c0c,         // .llvm.lto for fat LTO.
    AndroidRelr = 0x6fffff00,     // Relocation entries; only offsets.
    GnuAttributes = 0x6ffffff5,   // Object attributes.
    GnuHash = 0x6ffffff6,         // GNU-style hash table.
    GnuVerDef = 0x6ffffffd,       // GNU version definitions.
    GnuVerNeed = 0x6ffffffe,      // GNU version references.
    GnuVerSym = 0x6fffffff,       // GNU symbol versions table.

    // Start of arch-specific types
    HexOrdered = 0x70000000, // Link editor is to sort the entries in this section based on their sizes
    Proc1 = 0x70000001, // ARM: Exception Index table, X86_64: Unwind Information, CSKY: Attributes
    Proc2 = 0x70000002, // ARM: BPABI DLL dynamic linking pre-emption map
    ProcAttr = 0x70000003, // Object file compatibility attributes, used by ARM, MSP430, and RISCV
    Proc4 = 0x70000004, // ARM: DebugOverlay, AARCH64: Auth Relr
    ArmOverlaySection = 0x70000005,
    MipsRegInfo = 0x70000006, // Register usage information
    Aarch64MemTagGlobalsStatic = 0x70000007,
    Aarch64MemTagGlobalsDynamic = 0x70000008,
    MipsOptions = 0x7000000d,  // General options
    MipsDwarf = 0x7000001e,    // DWARF debugging section.
    MipsAbiFlags = 0x7000002a, // ABI information.

    // Values above 0x80000000 are reserved for user specific types
    Unknown = 0x87654321,
}
