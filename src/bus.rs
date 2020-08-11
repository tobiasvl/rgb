
use crate::PPU;

pub struct Bus {
  pub bootrom: [u8; 256],
  pub cartridge: MBC,
  pub ppu: PPU,
  pub wram: [u8; 0x2000], // TODO banks
  pub hram: [u8; 127],
  pub bootrom_enabled: bool,
  pub interrupt_enable: u8,
  pub interrupt_flags: u8,
  pub serial: u8,
  pub serial_control: u8
}

impl Bus {
  pub fn read_byte(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x00FF => if self.bootrom_enabled {
          self.bootrom[address as usize]
        } else {
          self.cartridge.read_byte(address)
        },
      0x0100..=0x3FFF => self.cartridge.read_byte(address),
      0x4000..=0x7FFF => self.cartridge.read_byte(address),
      0x8000..=0x9FFF => self.ppu.vram[(address - 0x8000) as usize],
      0xA000..=0xBFFF => self.cartridge.read_byte(address - 0xA000),
      0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],
      0xD000..=0xDFFF => self.wram[(address - 0xD000) as usize],
      0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
      0xFE00..=0xFE9F => self.ppu.oam[(address - 0xFE00) as usize],
      0xFEA0..=0xFEFF => 0x00,
      0xFF01 => self.serial,
      0xFF02 => self.serial_control,
      0xFF42 => self.ppu.scy,
      0xFF44 => 0x90, // TODO hardcoded LY
      0xFF00..=0xFF7F => 0x00,
      0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
      0xFFFF => self.interrupt_enable 
    }
  }

  pub fn read_word(&self, address: u16) -> u16 {
    (self.read_byte(address + 1) as u16) << 8 | self.read_byte(address) as u16
  }

  pub fn write_byte(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x00FF => if !self.bootrom_enabled { self.cartridge.write_byte(address, value) },
      0x0100..=0x3FFF => self.cartridge.write_byte(address, value),
      0x4000..=0x7FFF => self.cartridge.write_byte(address - 0x4000, value),
      0x8000..=0x9FFF => self.ppu.vram[(address - 0x8000) as usize] = value,
      0xA000..=0xBFFF => self.cartridge.write_byte(address - 0xA000, value),
      0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize] = value,
      0xD000..=0xDFFF => self.wram[(address - 0xD000) as usize] = value,
      0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
      0xFE00..=0xFE9F => self.ppu.oam[(address - 0xFE00) as usize] = value,
      0xFF01 => self.serial = value,
      0xFF02 => self.serial_control = value,
      0xFF42 => self.ppu.scy = value,
      0xFF50 => if value > 0 { self.bootrom_enabled = false },
      0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
      0xFFFF => self.interrupt_enable = value,
      _ => ()
    }
  }

  pub fn write_word(&self, address: u16) -> u16 {
    (self.read_byte(address + 1) as u16) << 8 | self.read_byte(address) as u16
  }
}

pub enum MBCKind {
  NoMBC,
  MBC1
}

pub struct MBC {
  pub kind: MBCKind,
  pub rom: [u8; 0x8000],
  pub ram: [u8; 0x2000],
  pub active_bank: u8,
  pub ram_enabled: bool
}

impl MBC {
  fn read_byte(&self, address: u16) -> u8 {
    match self.kind {
      MBCKind::NoMBC => self.rom[address as usize],
      MBCKind::MBC1 => match address {
        0x0000..=0x3FFF => self.rom[address as usize],
        0x4000..=0x7FFF => self.rom[(address * self.active_bank as u16) as usize],
        0xA000..=0xBFFF => if self.ram_enabled { self.ram[(address - 0xA000) as usize] } else { 0xFF },
        _ => 0xFF
      }
    }
  }
  
  fn write_byte(&mut self, address: u16, value: u8) {
    match self.kind {
      MBCKind::NoMBC => (),
      MBCKind::MBC1 => match address {
        0x0000..=0x1FFF => self.ram_enabled = value & 0x0A > 0,
        0x2000..=0x3FFF => match value & 0x1F {
          0x00 | 0x20 | 0x40 | 0x60 => self.active_bank = value + 1,
          _ => self.active_bank = value
        },
        _ => ()
      }
    }
  }
}