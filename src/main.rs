mod cpu;
mod bus;
mod ppu;

use cpu::{CPU,Registers,Flags};
use bus::{Bus,MBC,MBCKind};
use ppu::PPU;

fn main() {
  let mut cpu = CPU {
    bus: Bus {
      bootrom_enabled: true,
      cartridge: MBC {
        kind: MBCKind::NoMBC,
        rom: [0; 32768]
      },
      bootrom: [0; 256],
      wram: [0; 0x2000],
      hram: [0; 127],
      ppu: PPU {
        vram: [0; 8192],
        oam: [0; 0xA0],
      },
      ie: false
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
    }
  };

  let bootrom = std::fs::read("boot.gb")
    .expect("Something went wrong");

  for address in 0..256 {
    cpu.bus.bootrom[address] = bootrom[address];
  }

  loop {
    print!("{:X}: ", cpu.registers.pc);
    let opcode = cpu.fetch();
    println!("{:X}", opcode);
    let instruction = cpu.decode(opcode);
    cpu.execute(instruction);
  }
}