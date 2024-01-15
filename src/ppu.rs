use crate::interrupts::Interrupt;

pub struct Ppu {
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    pub scy: u8,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            scy: 0,
        }
    }
}

impl Ppu {
    pub(crate) fn tick(&mut self) -> Option<Interrupt> {
        None
    }
}
