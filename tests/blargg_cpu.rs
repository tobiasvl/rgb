#![allow(clippy::unwrap_used)]
use rgb_emu::bus::*;
use rgb_emu::cpu::*;
use rgb_emu::ppu::*;

#[test]
fn blargg_cpu_all() -> Result<(), String> {
    run_blargg_cpu_test("cpu_instrs.gb")
}

#[test]
fn blargg_cpu_01() -> Result<(), String> {
    run_blargg_cpu_test("individual/01-special.gb")
}

#[test]
fn blargg_cpu_02() -> Result<(), String> {
    run_blargg_cpu_test("individual/02-interrupts.gb")
}

#[test]
fn blargg_cpu_03() -> Result<(), String> {
    run_blargg_cpu_test("individual/03-op sp,hl.gb")
}

#[test]
fn blargg_cpu_04() -> Result<(), String> {
    run_blargg_cpu_test("individual/04-op r,imm.gb")
}

#[test]
fn blargg_cpu_05() -> Result<(), String> {
    run_blargg_cpu_test("individual/05-op rp.gb")
}

#[test]
fn blargg_cpu_06() -> Result<(), String> {
    run_blargg_cpu_test("individual/06-ld r,r.gb")
}

#[test]
fn blargg_cpu_07() -> Result<(), String> {
    run_blargg_cpu_test("individual/07-jr,jp,call,ret,rst.gb")
}

#[test]
fn blargg_cpu_08() -> Result<(), String> {
    run_blargg_cpu_test("individual/08-misc instrs.gb")
}

#[test]
fn blargg_cpu_09() -> Result<(), String> {
    run_blargg_cpu_test("individual/09-op r,r.gb")
}

#[test]
fn blargg_cpu_10() -> Result<(), String> {
    run_blargg_cpu_test("individual/10-bit ops.gb")
}

#[test]
fn blargg_cpu_11() -> Result<(), String> {
    run_blargg_cpu_test("individual/11-op a,(hl).gb")
}

fn run_blargg_cpu_test(path: &str) -> Result<(), String> {
    let mut cpu = Cpu {
        bus: Bus {
            bootrom_enabled: false,
            cartridge: Mbc {
                kind: MBCKind::NoMBC,
                rom: [0; 0x8000],
                ram: [0; 0x2000],
                ram_enabled: false,
                active_bank: 0,
            },
            bootrom: [0; 256],
            wram: [0; 0x2000],
            hram: [0; 127],
            ppu: Ppu {
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

    let rom = std::fs::read(String::from("tests/gb-test-roms/cpu_instrs/") + path)
        .expect("Unable to open ROM");

    cpu.bus.cartridge.rom[..].clone_from_slice(&rom[..]);

    let mut serial_output: String = String::new();

    loop {
        // TODO check for interrupts
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
        if cpu.bus.read_byte(0xFF02) != 0 {
            let character = cpu.bus.read_byte(0xFF01) as char;
            if character == '\n' {
                if serial_output.ends_with("Passed") {
                    return Ok(());
                } else if serial_output.lines().last().unwrap().starts_with("Failed") {
                    return Err(serial_output);
                }
            }
            serial_output.push(character);
            cpu.bus.write_byte(0xFF02, 0);
        }
    }
}
