use clap::Parser;
use std::path::PathBuf;

mod bus;
mod cartridge;
mod cpu;
mod ppu;

use cpu::{Cpu, RegisterPair};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Game Boy ROM file
    #[arg(index = 1, value_name = "ROM")]
    rom: PathBuf,

    /// Game Boy Boot ROM file
    #[arg(short, long, value_name = "FILE")]
    bootrom: Option<PathBuf>,

    /// Log debugging information to stdout
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut cpu = Cpu::new();

    if !match cli.bootrom {
        Some(bootrom_file) => match std::fs::read(bootrom_file) {
            Ok(bootrom) => {
                cpu.bus.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);
                cpu.bus.bootrom_enabled = true;
                true
            }
            Err(_) => {
                println!("Can't open boot ROM file, skipping...");
                false
            }
        },
        None => false,
    } {
        cpu.registers.pc = 0x100;
        cpu.registers.a = 0x01;
        cpu.registers.b = 0x00;
        cpu.registers.c = 0x13;
        cpu.registers.d = 0x00;
        cpu.registers.e = 0xD8;
        cpu.registers.h = 0x01;
        cpu.registers.l = 0x4D;
        cpu.flags.z = true;
        cpu.flags.n = false;
        cpu.flags.h = true;
        cpu.flags.c = true;
        cpu.registers.sp = 0xFFFE;
    };

    let rom = std::fs::read(cli.rom).expect("Unable to open ROM");
    cpu.bus.cartridge = Some(cartridge::from_rom(rom));

    loop {
        // gucci:
        if cli.debug {
            println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
                cpu.registers.a,
                cpu.get_register_pair(&RegisterPair::AF) & 0xFF,
                cpu.registers.b,
                cpu.registers.c,
                cpu.registers.d,
                cpu.registers.e,
                cpu.registers.h,
                cpu.registers.l,
                cpu.get_register_pair(&RegisterPair::SP),
                cpu.registers.pc,
                cpu.bus.read_byte(cpu.registers.pc),
                cpu.bus.read_byte(cpu.registers.pc+1),
                cpu.bus.read_byte(cpu.registers.pc+2),
                cpu.bus.read_byte(cpu.registers.pc+3),
            );
        }
        // TODO check for interrupts
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
    }
}
