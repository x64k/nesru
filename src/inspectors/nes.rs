/* iNES format decoder */

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::convert::TryFrom;

use mmapio;

use crate::inspectors::AnalysisContext;
use crate::inspectors::ROMInfo;
use crate::inspectors::MirroringType;
use crate::inspectors::BankType;
use crate::inspectors::BankInfo;

const INES_PRG_SIZE_IDX: usize = 0x04;
const INES_CHR_SIZE_IDX: usize = 0x05;
const INES_FLAGS_1_IDX:  usize = 0x06;

const INES_MIRRORING_TYPE_FOUR_SCREEN_MASK: u8 = 0x08;
const INES_MIRRORING_TYPE_HV_MASK: u8 = 0x01;
const INES_TRAINER_PRESENT_MASK: u8 = 0x04;

const INES_PRG_BANK_SIZE: u32 = 16 * 1024;

const INES_PRG_TRAINER_SIZE: u32 = 512;
const INES_HDR_SIZE: u32 = 16;

pub struct NESContext {
    // ROM file path
    filepath: String,
    // Disk-backed file
    file: std::fs::File,
    // File's memory map
    map: mmapio::Mmap,
    // Empty byte value
    empty: u8,
}

impl NESContext {
    fn get_prg_size(&self) -> u8 {
        return self.map[INES_PRG_SIZE_IDX];
    }

    fn get_chr_size(&self) -> u8 {
        return self.map[INES_CHR_SIZE_IDX];
    }

    /* TODO: Disambiguate nametable mirrorring settings. E.g. UNROM
     * 512 has bit 3 (FOUR_SCREEN) set to 1 and the LSB set to 0 to
     * indicate a one-screen board, and 1..1 to indicate a 4-screen
     * board. */
    fn get_mirroring_type(&self) -> Option<MirroringType> {
        if (self.map[INES_FLAGS_1_IDX] & INES_MIRRORING_TYPE_FOUR_SCREEN_MASK) != 0 {
            return Some(MirroringType::FourScreen);
        } else {
            if (self.map[INES_FLAGS_1_IDX] & INES_MIRRORING_TYPE_HV_MASK) != 0 {
                return Some(MirroringType::Horizontal);
            } else {
                return Some(MirroringType::Vertical);
            }
        }
    }

    fn trainer_present(&self) -> bool {
        self.map[INES_FLAGS_1_IDX] & INES_TRAINER_PRESENT_MASK != 0
    }
}

pub fn matches(hdr: &[u8; 16]) -> bool {
    if hdr[0] == 0x4E && hdr[1] == 0x45 && hdr[2] == 0x53 && hdr[3] == 0x1A {
        if hdr[7] & 0x0C == 0x08 {
            return false;
        } else {
            return true;
        }
    } else {
        return false;
    }
}

pub fn new(path: &String, e: u8) -> Result<NESContext, std::io::Error> {
    let f = File::open(path)?;
    let mmap = unsafe { mmapio::MmapOptions::new().map(&f)? };

    return Ok(NESContext {
        filepath: path.clone(),
        file: f,
        map: mmap,
        empty: e,
    });
}

impl AnalysisContext for NESContext {
    fn hdr_analysis(&self) -> ROMInfo {
        let len: Option<u64> = match File::metadata(&self.file) {
            Ok(m) => Some(m.len()),
            Err(_) => None
        };

        ROMInfo {
            filename:  String::from(self.filepath.clone()),
            filesize:  len,
            prgsize:   self.get_prg_size(),
            chrsize:   self.get_chr_size(),
            mirroring: self.get_mirroring_type(),
        }
    }

    fn bank_analysis(&self, bnum: u8, btype: BankType) -> Result<BankInfo, std::io::Error> {
        let bstart = if self.trainer_present() {
            INES_HDR_SIZE + INES_PRG_TRAINER_SIZE + (u32::from(bnum) * INES_PRG_BANK_SIZE)
        } else {
            INES_HDR_SIZE + (u32::from(bnum) * INES_PRG_BANK_SIZE)
        };

        let bend = u32::from(bstart) + INES_PRG_BANK_SIZE;

        let blen = match usize::try_from(bstart + bend) {
            Ok(e) => e,
            Err(_) => return Err(Error::new(ErrorKind::UnexpectedEof, "Invalid bank number")),
        };

        if blen > self.map.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Invalid bank number"));
        }

        let mut empty = 0;
        for i in bstart as usize..bend as usize {
            if self.map[i] == self.empty {
                empty += 1;
            }
        }
            
        Ok(BankInfo {
            banktype: Some(btype),
            size: (bend - bstart),
            freespace: empty,
        })
    }
}
