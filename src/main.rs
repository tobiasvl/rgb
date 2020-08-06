mod cpu;
mod bus;
mod ppu;

use cpu::{CPU,Registers,Flags,RegisterPair};
use bus::{Bus,MBC,MBCKind};
use ppu::PPU;

fn main() {
  let mut cpu = CPU {
    bus: Bus {
      bootrom_enabled: true,
      cartridge: MBC {
        kind: MBCKind::NoMBC,
        rom: [0; 32768],
        ram: [0; 0x2000],
        ram_enabled: false,
        active_bank: 0
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
      interrupt_flags: 0
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
      l: 0
    },
    flags: Flags {
      z: false,
      n: false,
      h: false,
      c: false
    },
    ime: false,
  };

  let bootrom = std::fs::read("boot.gb")
    .expect("Something went wrong");

  cpu.bus.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);

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
    //print!("{:X}: ", cpu.registers.pc);
    let opcode = cpu.fetch();
    //println!("{:X}", opcode);
    let instruction = cpu.decode(opcode);
    cpu.execute(instruction);
  }
}