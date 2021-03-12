use std::fmt;
pub mod nes;

#[derive(Copy, Clone)]
pub enum MirroringType {
    Horizontal,
    Vertical,
    FourScreen,
}

impl fmt::Display for MirroringType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MirroringType::Horizontal =>
                f.write_str("Horizontal mirroring (vertical arrangement)"),
            MirroringType::Vertical =>
                f.write_str("Vertical mirroring (horizontal arrangement)"),
            MirroringType::FourScreen =>
                f.write_str("Four-screen VRAM"),
        }
    }
}

pub struct ROMInfo {
    pub filename: String,
    pub filesize: Option<u64>,
    pub prgsize: u8,
    pub chrsize: u8,
    // TODO:
    // Mapper, mirroring, battery, trainer
    // Mapper, VS/Playchoice
    // PRG-RAM size
    // TV system
    // TV system PRG-RAM presence
    // Ripper name, if present -- generally 0
    pub mirroring: Option<MirroringType>,
}

#[derive(Copy, Clone)]
pub enum BankType {
    Prg,
    Chr,
    PCInstRom,
    PCRom,
}

impl fmt::Display for BankType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BankType::Prg => f.write_str("PRG"),
            BankType::Chr => f.write_str("CHR"),
            BankType::PCInstRom => f.write_str("PlayChoice INST-ROM"),
            BankType::PCRom => f.write_str("PlayChoice PROM"),
        }
    }
}

pub struct BankInfo {
    // Bank type (PRG/CHR/PlayChoice INST/PlayChoice PROM)
    pub banktype: Option<BankType>,
    // Bank size, in bytes
    pub size: u32,
    // Free space, approx., in bytes
    pub freespace: u32,
}

pub trait AnalysisContext {
    // Grab basic information about the header
    fn hdr_analysis(&self) -> ROMInfo;
    // Grab information about a given bank
    fn bank_analysis(&self, bnum: u8, btype: BankType) -> Result<BankInfo, std::io::Error>;
}
