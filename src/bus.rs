
use crate::PPU;
pub struct Bus {
  pub bootrom: [u8; 256],
  pub cartridge: MBC,
  pub ppu: PPU,
  pub wram: [u8; 0x2000], // TODO banks
  pub hram: [u8; 127],
  pub ie: bool,
  pub bootrom_enabled: bool
}

impl Bus {
  pub fn fetch_byte(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x00FF => if self.bootrom_enabled {
          self.bootrom[address as usize]
        } else {
          self.cartridge.read_byte(address)
        },
      0x0100..=0x3FFF => self.cartridge.read_byte(address),
      0x4000..=0x7FFF => self.cartridge.read_byte(address - 0x4000),
      0x8000..=0x9FFF => self.ppu.vram[(address - 0x8000) as usize],
      0xA000..=0xBFFF => self.cartridge.read_byte(address - 0xA000),
      0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],
      0xD000..=0xDFFF => self.wram[(address - 0xD000) as usize],
      0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
      0xFE00..=0xFE9F => self.ppu.oam[(address - 0xFE00) as usize],
      0xFEA0..=0xFEFF => 0x00,
      0xFF00..=0xFF7F => 0x00,
      0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
      0xFFFF => if self.ie { 1 } else { 0 }
    }
  }

  pub fn fetch_word(&self, address: u16) -> u16 {
    return (self.fetch_byte(address) as u16) << 8 | self.fetch_byte(address + 1) as u16;
  }
}

pub enum MBCKind {
  NoMBC,
}

pub struct MBC {
  pub kind: MBCKind,
  pub rom: [u8; 0x8000],
}

impl MBC {
  fn read_byte(&self, address: u16) -> u8 {
    match self.kind {
      MBCKind::NoMBC => return self.rom[address as usize]
    }
  }
}