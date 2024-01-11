pub trait Cartridge {
    #[must_use]
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);
}

/// # Panics
///
/// Will panic if cartridge header is malformed or not present
#[must_use]
#[allow(clippy::similar_names)]
pub fn from_rom(rom: Vec<u8>) -> Box<dyn Cartridge> {
    let header_rom_size = rom
        .get(0x0148)
        .expect("Unable to find ROM size in cartridge header");
    assert!(*header_rom_size <= 8);
    let rom_size = (2_u32).pow(15 + u32::from(*header_rom_size)) as usize;
    assert!(rom_size == rom.len());

    let ram: Option<Vec<u8>> = if let Some(header_ram_size) = rom.get(0x0149) {
        match header_ram_size {
            0x00 => None,
            0x02 => Some(Vec::with_capacity(0x2000)),
            0x03 => Some(Vec::with_capacity(0x8000)),
            0x04 => Some(Vec::with_capacity(0x20000)),
            0x05 => Some(Vec::with_capacity(0x10000)),
            _ => panic!("Unknown RAM size in cartridge header"),
        }
    } else {
        panic!("Unable to find RAM size in cartridge header");
    };
    if let Some(header_mbc) = rom.get(0x0147) {
        match header_mbc {
            0x00 => Box::new(NoMbc { rom, ram }), // TODO assert that ROM is 32 KiB?
            0x01 => Box::new(Mbc1 {
                // TODO assert that RAM/ROM combination is correct?
                rom,
                ram,
                ..Default::default()
            }),
            _ => panic!("Unknown MBC in cartridge header"),
        }
    } else {
        panic!("Unable to find MBC in cartridge header")
    }
}

pub struct JsMoo {
    pub ram: [u8; 0x10000],
}

impl Cartridge for JsMoo {
    fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }
}

pub struct NoMbc {
    pub rom: Vec<u8>,
    pub ram: Option<Vec<u8>>,
}

impl Cartridge for NoMbc {
    fn read_byte(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write_byte(&mut self, _address: u16, _value: u8) {}
}

#[derive(Default)]
pub struct Mbc1 {
    pub rom: Vec<u8>,
    pub ram: Option<Vec<u8>>,
    pub active_bank: u8,
    pub ram_enabled: bool,
}

impl Cartridge for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let active_bank = match self.active_bank {
                    0x00 | 0x20 | 0x40 | 0x60 => self.active_bank + 1,
                    _ => self.active_bank,
                };
                self.rom[(address * u16::from(active_bank)) as usize]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled || self.ram.is_none() {
                    0xFF
                } else {
                    self.ram.as_ref().unwrap()[(address - 0xA000) as usize]
                }
            }
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value & 0x0A > 0,
            0x2000..=0x3FFF => match value & 0x1F {
                0x00 | 0x20 | 0x40 | 0x60 => self.active_bank = value + 1,
                _ => self.active_bank = value,
            },
            _ => (),
        }
    }
}
