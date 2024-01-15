#[derive(Debug)]
pub enum Interrupt {
    VBlank = 0,
    Stat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}
