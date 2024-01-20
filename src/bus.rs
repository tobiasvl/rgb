use crate::cartridge::Cartridge;
use crate::interrupts::Interrupt;
use crate::ppu::Ppu;
use crate::timer::Timer;

pub trait Bus {
    fn tick(&mut self);
    fn read_byte(&mut self, address: u16) -> u8;
    fn peek_byte(&self, address: u16) -> u8;
    fn read_word(&mut self, address: u16) -> u16;
    fn write_byte(&mut self, address: u16, value: u8);
    fn write_word(&mut self, address: u16, value: u16);
    fn set_post_boot_state(&mut self);
    fn get_interrupt_enable(&self) -> u8;
    fn set_interrupt_enable(&mut self, value: u8);
    fn get_interrupt_flags(&self) -> u8;
    fn set_interrupt_flags(&mut self, flags: u8);
    fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>);
    fn remove_cartridge(&mut self);
    fn set_boot_rom(&mut self, bootrom: Vec<u8>);
}

pub struct DmgBus {
    pub bootrom: [u8; 256],
    pub ppu: Ppu,
    pub wram: [u8; 0x2000], // TODO banks
    pub hram: [u8; 127],
    pub bootrom_enabled: bool,
    pub interrupt_enable: u8,
    pub interrupt_flags: u8,
    pub serial: u8,
    pub serial_control: u8,
    pub(crate) timer: Timer,
    pub cartridge: Option<Box<dyn Cartridge>>,
}

impl Default for DmgBus {
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
            timer: Timer::default(),
            cartridge: None,
            bootrom_enabled: false,
        }
    }
}

impl DmgBus {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Bus for DmgBus {
    /// Tick one M-cycle (4 T-cycles)
    fn tick(&mut self) {
        if let Some(irq) = self.ppu.tick() {
            match irq {
                Interrupt::VBlank => self.interrupt_flags |= 1,
                Interrupt::Stat => self.interrupt_flags |= 2,
                _ => unreachable!(),
            }
        }
        if let Some(Interrupt::Timer) = self.timer.tick() {
            self.interrupt_flags |= 4;
        }
    }

    fn set_boot_rom(&mut self, bootrom: Vec<u8>) {
        self.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);
        self.bootrom_enabled = true;
    }

    fn peek_byte(&self, address: u16) -> u8 {
        #[allow(clippy::match_overlapping_arm)]
        if self.bootrom_enabled && (0x000..0x100).contains(&address) {
            self.bootrom[address as usize]
        } else {
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
                0xFF04..=0xFF07 => self.timer.read_byte(address),
                0xFF42 => self.ppu.scy,
                0xFF44 => 0x90, // TODO hardcoded LY
                0xFF0F => self.interrupt_flags,
                0xFF00..=0xFF7F => 0x00,
                0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
                0xFFFF => self.interrupt_enable,
            }
        }
    }

    #[must_use]
    fn read_byte(&mut self, address: u16) -> u8 {
        let byte = self.peek_byte(address);
        self.tick();
        byte
    }

    #[must_use]
    fn read_word(&mut self, address: u16) -> u16 {
        let low_byte = u16::from(self.read_byte(address));
        u16::from(self.read_byte(address + 1)) << 8 | low_byte
    }

    fn write_byte(&mut self, address: u16, value: u8) {
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
            0xFF04..=0xFF07 => self.timer.write_byte(address, value),
            0xFF0F => self.interrupt_flags = 0xE0 | value,
            0xFF42 => self.ppu.scy = value,
            0xFF50 => {
                if value > 0 {
                    self.bootrom_enabled = false;
                }
            }
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupt_enable = 0xE0 | value,
            _ => (),
        }

        self.tick();
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
    }

    fn set_post_boot_state(&mut self) {
        self.timer.sysclock = 0xAB;
    }

    fn get_interrupt_enable(&self) -> u8 {
        self.interrupt_enable
    }

    fn set_interrupt_enable(&mut self, flags: u8) {
        self.interrupt_enable = flags;
    }

    fn get_interrupt_flags(&self) -> u8 {
        self.interrupt_flags
    }

    fn set_interrupt_flags(&mut self, flags: u8) {
        self.interrupt_flags = flags;
    }

    fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = Some(cartridge);
    }

    fn remove_cartridge(&mut self) {
        self.cartridge = None;
    }
}
