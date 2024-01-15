use crate::interrupts::Interrupt;

#[derive(Default)]
pub struct Timer {
    pub(crate) sysclock: u16,
    pub(crate) tima: u8,
    pub(crate) tma: u8,
    edge: bool,
    pub(crate) tima_enable: bool,
    pub(crate) clock_select: u8,
}

impl Timer {
    pub fn tick(&mut self) -> Option<Interrupt> {
        self.sysclock = self.sysclock.wrapping_add(4);

        if self.tima_enable {
            let old_edge = self.edge;
            self.edge = (self.sysclock
                >> match self.clock_select {
                    0 => 9,
                    1 => 3,
                    2 => 5,
                    3 => 7,
                    _ => unreachable!(),
                }
                & 1)
                != 0;
            if !self.edge && old_edge {
                let increment = self.tima.overflowing_add(1);
                if increment.1 {
                    self.tima = self.tma;
                    return Some(Interrupt::Timer);
                }
                self.tima = increment.0;
            }

            //self.tima = increment.0;
        }
        None
    }

    #[must_use]
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.sysclock >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => 0xF8 | (u8::from(self.tima_enable) << 2) | self.clock_select,
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.sysclock = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                self.tima_enable = value & 4 != 0;
                self.clock_select = value & 3;
            }
            _ => unreachable!(),
        }
    }
}
