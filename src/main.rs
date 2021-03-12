use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

mod inspectors;
use inspectors::AnalysisContext;

fn print_usage(pname: &String) {
    std::println!("Usage: {} <file>", pname);
}

fn get_header(filepath: &String) -> Result<[u8; 16], io::Error> {
    let path = Path::new(filepath);
    let mut f = File::open(&path)?;
    let mut hdrbuf = [0; 16];

    f.read(&mut hdrbuf)?;

    return Ok(hdrbuf);
}

fn print_summary(s: &inspectors::ROMInfo) {
    std::println!("File: {}", s.filename);

    match s.filesize {
        Some(size) => {
            std::println!("File size: {}K ({} bytes)", size / 1024, size);
        },
        None => {
            std::println!("File size: unknown");
        }
    }

    std::println!("PRG ROM banks: {}x16K banks ({}K, {} bytes)",
                  s.prgsize, s.prgsize * 16, u32::from(s.prgsize) * 16 * 1024);

    if s.chrsize == 0 {
        std::println!("CHR ROM: N/A (board uses CHR RAM)");
    } else {
        std::println!("CHR ROM: {}x8K banks ({}K, {} bytes)",
                      s.chrsize, s.chrsize * 8, u32::from(s.chrsize) * 8 * 1024);
    }

    match s.mirroring {
        Some(m) => std::println!("Mirroring: {}", m),
        None => std::println!("Mirroring: Unknown"),
    }
}

fn print_bank_usage(banknum: u8, banktype: inspectors::BankType, ctx: &dyn inspectors::AnalysisContext) {
    match ctx.bank_analysis(banknum, banktype) {
        Ok(bankinfo) => {
            let fs_perc = 100.0 * (f64::from(bankinfo.freespace))/f64::from(bankinfo.size);
            std::println!("{} {}: {}/{} bytes ({:.2}%)", banktype, banknum, bankinfo.freespace, bankinfo.size, fs_perc);
        },
        Err(_) => {
            std::println!("Analysis error");
        }
    }
}

fn do_bank_usage(ctx: &dyn inspectors::AnalysisContext, hinfo: inspectors::ROMInfo) {
    for rom in 0..hinfo.prgsize {
        print_bank_usage(rom, inspectors::BankType::Prg, ctx);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    let hdr = match get_header(&args[1]) {
        Ok(h) => h,
        Err(e) => {
            std::println!("Couldn't get file information: {}", e);
            return;
        }
    };

    let ctx =  if inspectors::nes::matches(&hdr) {
            match inspectors::nes::new(&args[1], 0x00) {
                Ok(c) => c,
                Err(e) => {
                    std::println!("Analysys failed: {}", e);
                    return;
                }
            }
        } else {
            std::println!("Unknown ROM format.");
            return;
        };

    let hinfo = ctx.hdr_analysis();
    print_summary(&hinfo);
    do_bank_usage(&ctx, hinfo);
    
}
