use crate::cartridge::Cartridge;
use crate::ppu::Ppu;

pub struct Bus {
    pub bootrom: [u8; 256],
    pub ppu: Ppu,
    pub wram: [u8; 0x2000], // TODO banks
    pub hram: [u8; 127],
    pub bootrom_enabled: bool,
    pub interrupt_enable: u8,
    pub interrupt_flags: u8,
    pub serial: u8,
    pub serial_control: u8,
    pub cartridge: Option<Box<dyn Cartridge>>,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            bootrom: [0; 256],
            wram: [0; 0x2000],
            hram: [0; 127],
            ppu: Ppu::default(),
            interrupt_enable: 0,
            interrupt_flags: 0,
            serial: 0,
            serial_control: 0,
            cartridge: None,
            bootrom_enabled: false,
        }
    }
}

impl Bus {
    #[must_use]
    pub fn read_byte(&self, address: u16) -> u8 {
        #[allow(clippy::match_overlapping_arm)]
        if self.bootrom_enabled && (0x000..0x100).contains(&address) {
            return self.bootrom[address as usize];
        }
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => {
                if let Some(cartridge) = &self.cartridge {
                    cartridge.read_byte(address)
                } else {
                    0xFF
                }
            }
            0x8000..=0x9FFF => self.ppu.vram[(address - 0x8000) as usize],
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.ppu.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0x00,
            0xFF01 => self.serial,
            0xFF02 => self.serial_control,
            0xFF42 => self.ppu.scy,
            0xFF44 => 0x90, // TODO hardcoded LY
            0xFF00..=0xFF7F => 0x00,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupt_enable,
        }
    }

    #[must_use]
    pub fn read_word(&self, address: u16) -> u16 {
        u16::from(self.read_byte(address + 1)) << 8 | u16::from(self.read_byte(address))
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => {
                // TODO What happens when writing here while the boot ROM is mapped?
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write_byte(address, value);
                }
            }
            0x8000..=0x9FFF => self.ppu.vram[(address - 0x8000) as usize] = value,
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFE00..=0xFE9F => self.ppu.oam[(address - 0xFE00) as usize] = value,
            0xFF01 => self.serial = value,
            0xFF02 => self.serial_control = value,
            0xFF42 => self.ppu.scy = value,
            0xFF50 => {
                if value > 0 {
                    self.bootrom_enabled = false;
                }
            }
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupt_enable = value,
            _ => (),
        }
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
    }
}
