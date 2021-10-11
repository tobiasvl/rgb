extern crate clap;
use clap::{App, Arg};

mod bus;
mod cpu;
mod ppu;

use bus::{Bus, MBCKind, MBC};
use cpu::{Flags, RegisterPair, Registers, CPU};
use ppu::PPU;

fn main() {
    let matches = App::new("RGB")
        .version("1.0")
        .author("Tobias V. Langhoff <tobias@langhoff.no>")
        .about("Game Boy emulator")
        .arg(
            Arg::with_name("ROM")
                .help("ROM file")
                .required(true) // for the time being
                .index(1),
        )
        .arg(
            Arg::with_name("bootrom")
                .short("b")
                .long("bootrom")
                .help("Boot ROM file")
                .required(false),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .takes_value(false)
                .help("Debug log"),
        )
        .get_matches();

    let mut cpu = CPU {
        bus: Bus {
            bootrom_enabled: false,
            cartridge: MBC {
                kind: MBCKind::NoMBC,
                rom: [0; 32768],
                ram: [0; 0x2000],
                ram_enabled: false,
                active_bank: 0,
            },
            bootrom: [0; 256],
            wram: [0; 0x2000],
            hram: [0; 127],
            ppu: PPU {
                vram: [0; 8192],
                oam: [0; 0xA0],
                scy: 0,
            },
            interrupt_enable: 0,
            interrupt_flags: 0,
            serial: 0,
            serial_control: 0,
        },
        registers: Registers {
            sp: 0,
            pc: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        },
        flags: Flags {
            z: false,
            n: false,
            h: false,
            c: false,
        },
        ime: false,
    };

    if !match matches.value_of("bootrom") {
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

    let rom = std::fs::read(matches.value_of("ROM").unwrap())
        //let testrom = std::fs::read("gb-test-roms/cpu_instrs/individual/06-ld r,r.gb")
        .expect("Couldn't load ROM");

    cpu.bus.cartridge.rom[..].clone_from_slice(&rom[..]);

    let mut serial_output: String = String::from("");

    let debug = matches.is_present("debug"); // = args.len() > 1 && args[1] == "debug";

    loop {
        //let s = format!("PC: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X} ({:02X}{:02X}), ({:02X} {:02X} {:02X} {:02X})",
        //  cpu.registers.pc,
        //  cpu.get_register_pair(&RegisterPair::AF),
        //  cpu.get_register_pair(&RegisterPair::BC),
        //  cpu.get_register_pair(&RegisterPair::DE),
        //  cpu.get_register_pair(&RegisterPair::HL),
        //  cpu.get_register_pair(&RegisterPair::SP),
        //  cpu.bus.read_byte(cpu.registers.sp),
        //  cpu.bus.read_byte(cpu.registers.sp+1),
        //  cpu.bus.read_byte(cpu.registers.pc),
        //  cpu.bus.read_byte(cpu.registers.pc+1),
        //  cpu.bus.read_byte(cpu.registers.pc+2),
        //  cpu.bus.read_byte(cpu.registers.pc+3),
        //);
        // gucci:
        if debug {
            println!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
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
        //println!("{} - {:?}", s, instruction);
        cpu.execute(instruction);
        if cpu.bus.read_byte(0xFF02) != 0 {
            let character = cpu.bus.read_byte(0xFF01) as char;
            if character == '\n' {
                println!("{}", serial_output);
                match &serial_output[..] {
                    "Passed" => std::process::exit(0),
                    "Failed" => std::process::exit(-1),
                    _ => serial_output = String::from(""),
                };
            } else {
                serial_output.push(character);
            }
            cpu.bus.write_byte(0xFF02, 0)
        }
    }
}

#[test]
fn test_initial_state_bootrom() {
    let mut cpu = CPU {
        bus: Bus {
            bootrom_enabled: false,
            cartridge: MBC {
                kind: MBCKind::NoMBC,
                rom: [0; 32768],
                ram: [0; 0x2000],
                ram_enabled: false,
                active_bank: 0,
            },
            bootrom: [0; 256],
            wram: [0; 0x2000],
            hram: [0; 127],
            ppu: PPU {
                vram: [0; 8192],
                oam: [0; 0xA0],
                scy: 0,
            },
            interrupt_enable: 0,
            interrupt_flags: 0,
            serial: 0,
            serial_control: 0,
        },
        registers: Registers {
            sp: 0,
            pc: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        },
        flags: Flags {
            z: false,
            n: false,
            h: false,
            c: false,
        },
        ime: false,
    };

    let bootrom = std::fs::read("boot.gb").expect("Test requires bootrom");
    cpu.bus.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);
    cpu.bus.bootrom_enabled = true;

    let testrom = std::fs::read("gb-test-roms/cpu_instrs/individual/06-ld r,r.gb")
        .expect("Test requires cartridge");
    cpu.bus.cartridge.rom[..].clone_from_slice(&testrom[..]);

    loop {
        println!("PC: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X} ({:02X}{:02X}), ({:02X} {:02X} {:02X} {:02X})",
      cpu.registers.pc,
      cpu.get_register_pair(&RegisterPair::AF),
      cpu.get_register_pair(&RegisterPair::BC),
      cpu.get_register_pair(&RegisterPair::DE),
      cpu.get_register_pair(&RegisterPair::HL),
      cpu.get_register_pair(&RegisterPair::SP),
      cpu.bus.read_byte(cpu.registers.sp),
      cpu.bus.read_byte(cpu.registers.sp+1),
      cpu.bus.read_byte(cpu.registers.pc),
      cpu.bus.read_byte(cpu.registers.pc+1),
      cpu.bus.read_byte(cpu.registers.pc+2),
      cpu.bus.read_byte(cpu.registers.pc+3),
    );
        // TODO check for interrupts
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
        if cpu.registers.pc == 0x100 {
            break;
        };
    }
    assert_eq!(cpu.registers.pc, 0x100);
    assert_eq!(cpu.registers.a, 0x01);
    assert_eq!(cpu.registers.b, 0x00);
    assert_eq!(cpu.registers.c, 0x13);
    assert_eq!(cpu.registers.d, 0x00);
    assert_eq!(cpu.registers.e, 0xD8);
    assert_eq!(cpu.registers.h, 0x01);
    assert_eq!(cpu.registers.l, 0x4D);
    assert_eq!(cpu.flags.z, true);
    assert_eq!(cpu.flags.n, false);
    assert_eq!(cpu.flags.h, true);
    assert_eq!(cpu.flags.c, true);
    assert_eq!(cpu.registers.sp, 0xFFFE);
}

#[test]
fn test_initial_state_bootrom_no_cart() {
    let mut cpu = CPU {
        bus: Bus {
            bootrom_enabled: false,
            cartridge: MBC {
                kind: MBCKind::NoMBC,
                rom: [0xFF; 32768],
                ram: [0; 0x2000],
                ram_enabled: false,
                active_bank: 0,
            },
            bootrom: [0; 256],
            wram: [0; 0x2000],
            hram: [0; 127],
            ppu: PPU {
                vram: [0; 8192],
                oam: [0; 0xA0],
                scy: 0,
            },
            interrupt_enable: 0,
            interrupt_flags: 0,
            serial: 0,
            serial_control: 0,
        },
        registers: Registers {
            sp: 0,
            pc: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        },
        flags: Flags {
            z: false,
            n: false,
            h: false,
            c: false,
        },
        ime: false,
    };

    let bootrom = std::fs::read("boot.gb").expect("Test requires bootrom");
    cpu.bus.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);
    cpu.bus.bootrom_enabled = true;

    loop {
        // TODO check for interrupts
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
        if cpu.registers.pc == 0xFA {
            break;
        };
    }
    assert_eq!(cpu.registers.pc, 0xFA);
    assert_eq!(cpu.registers.a, 0xFF); // TODO check this
    assert_eq!(cpu.registers.b, 0x00);
    assert_eq!(cpu.registers.c, 0x13);
    assert_eq!(cpu.registers.d, 0x00);
    assert_eq!(cpu.registers.e, 0xD8);
    assert_eq!(cpu.registers.h, 0x01);
    assert_eq!(cpu.registers.l, 0x4D);
    assert_eq!(cpu.flags.z, false);
    assert_eq!(cpu.flags.n, false);
    assert_eq!(cpu.flags.h, false); // TODO check this
    assert_eq!(cpu.flags.c, false);
    assert_eq!(cpu.registers.sp, 0xFFFE);
}
