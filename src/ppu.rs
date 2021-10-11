pub struct PPU {
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    pub scy: u8,
}
