use crate::bus::Bus;
use std::ops::{Index, IndexMut};

#[derive(Default)]
pub struct Cpu {
    pub registers: Registers,
    pub flags: Flags,
    pub ime: bool,
    pub bus: Bus,
}

impl Cpu {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Flags {
    pub z: bool,
    pub c: bool,
    pub n: bool,
    pub h: bool,
}

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Index<&Register> for Registers {
    type Output = u8;

    fn index(&self, index: &Register) -> &Self::Output {
        match index {
            Register::A => &self.a,
            Register::B => &self.b,
            Register::C => &self.c,
            Register::D => &self.d,
            Register::E => &self.e,
            Register::H => &self.h,
            Register::L => &self.l,
            _ => panic!("Unknown register {index:?}"),
        }
    }
}

impl IndexMut<&Register> for Registers {
    fn index_mut(&mut self, index: &Register) -> &mut Self::Output {
        match index {
            Register::A => &mut self.a,
            Register::B => &mut self.b,
            Register::C => &mut self.c,
            Register::D => &mut self.d,
            Register::E => &mut self.e,
            Register::H => &mut self.h,
            Register::L => &mut self.l,
            _ => panic!("Unknown register {index:?}"),
        }
    }
}

#[derive(Debug)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    IndirectHL,
    DecrementHL,
    IncrementHL,
    IndirectC,
}

#[derive(Debug)]
pub enum RegisterPair {
    BC,
    DE,
    HL,
    SP,
    AF,
}

fn inherent_condition_operand(opcode: u8) -> Condition {
    match opcode & 0o03 {
        0 => Condition::NonZero,
        1 => Condition::Zero,
        2 => Condition::NonCarry,
        3 => Condition::Carry,
        _ => panic!("This should never happen"),
    }
}

fn inherent_register_operand(opcode: u8) -> Register {
    match opcode & 0o07 {
        0 => Register::B,
        1 => Register::C,
        2 => Register::D,
        3 => Register::E,
        4 => Register::H,
        5 => Register::L,
        6 => Register::IndirectHL,
        7 => Register::A,
        _ => panic!("This should never happen"),
    }
}

fn inherent_registerpair_operand(opcode: u8) -> RegisterPair {
    match opcode & 0o07 {
        0 | 1 => RegisterPair::BC,
        2 | 3 => RegisterPair::DE,
        4 | 5 => RegisterPair::HL,
        6 | 7 => RegisterPair::SP,
        _ => panic!("This should never happen"),
    }
}

#[derive(Debug)]
pub enum Condition {
    Always,
    Zero,
    NonZero,
    Carry,
    NonCarry,
}

#[derive(Debug)]
pub enum Instruction {
    Ld(Operand, Operand),
    Xor(Operand),
    And(Operand),
    Add(Operand, Operand),
    Adc(Operand),
    Sub(Operand),
    Sbc(Operand),
    Or(Operand),
    Cp(Operand),
    Inc(Operand),
    Dec(Operand),
    Rlc(Register),
    Rrc(Register),
    Rl(Register),
    Rla,
    Rlca,
    Rr(Register),
    Rra,
    Rrca,
    Sla(Register),
    Sra(Register),
    Swap(Register),
    Srl(Register),
    Bit(u8, Register),
    Res(u8, Register),
    Set(u8, Register),
    Rst(u8),
    Ret(Condition),
    Reti,
    Jp(Condition, Operand),
    Jr(Condition, i8),
    Call(Condition, u16),
    Stop,
    Nop,
    Halt,
    Ei,
    Di,
    Push(RegisterPair),
    Pop(RegisterPair),
    Daa,
    Cpl,
    Scf,
    Ccf,
}

#[derive(Debug)]
pub enum Operand {
    Immediate8(u8),
    IndirectImmediate8(u8),
    Immediate16(u16),
    IndirectImmediate16(u16),
    StackOffset(i8),
    Register(Register),
    RegisterPair(RegisterPair),
    RegisterIndirect(RegisterPair),
}

impl Cpu {
    #[must_use]
    pub fn get_register_pair(&self, rp: &RegisterPair) -> u16 {
        match rp {
            RegisterPair::BC => (u16::from(self.registers.b) << 8) | u16::from(self.registers.c),
            RegisterPair::DE => (u16::from(self.registers.d) << 8) | u16::from(self.registers.e),
            RegisterPair::HL => (u16::from(self.registers.h) << 8) | u16::from(self.registers.l),
            RegisterPair::AF => {
                (u16::from(self.registers.a) << 8)
                    | if self.flags.z { 0x80 } else { 0 }
                    | if self.flags.n { 0x40 } else { 0 }
                    | if self.flags.h { 0x20 } else { 0 }
                    | if self.flags.c { 0x10 } else { 0 }
            }
            RegisterPair::SP => self.registers.sp,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn set_register_pair(&mut self, rp: &RegisterPair, value: u16) {
        match rp {
            RegisterPair::AF => {
                self.registers.a = (value >> 8) as u8;
                self.flags.z = value & 0x80 == 0x80;
                self.flags.n = value & 0x40 == 0x40;
                self.flags.h = value & 0x20 == 0x20;
                self.flags.c = value & 0x10 == 0x10;
            }
            RegisterPair::BC => {
                self.registers.b = (value >> 8) as u8;
                self.registers.c = value as u8;
            }
            RegisterPair::DE => {
                self.registers.d = (value >> 8) as u8;
                self.registers.e = value as u8;
            }
            RegisterPair::HL => {
                self.registers.h = (value >> 8) as u8;
                self.registers.l = value as u8;
            }
            RegisterPair::SP => self.registers.sp = value,
        }
    }

    fn fetch_imm8(&mut self) -> u8 {
        let value = self.bus.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        value
    }

    fn fetch_imm16(&mut self) -> u16 {
        let value = self.bus.read_word(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);
        value
    }

    pub fn fetch(&mut self) -> u8 {
        self.fetch_imm8()
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.bus.write_word(self.registers.sp, value);
    }

    fn pop(&mut self) -> u16 {
        let result = self.bus.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        result
    }

    #[allow(clippy::too_many_lines)]
    pub fn decode(&mut self, opcode: u8) -> Instruction {
        #[allow(clippy::match_overlapping_arm, clippy::cast_possible_wrap)]
        match opcode {
            0o00 => Instruction::Nop,
            0o01 | 0o21 | 0o41 | 0o61 => Instruction::Ld(
                Operand::RegisterPair(inherent_registerpair_operand(opcode >> 3)),
                Operand::Immediate16(self.fetch_imm16()),
            ),
            0o07 => Instruction::Rlca,
            0o17 => Instruction::Rrca,
            0o27 => Instruction::Rla,
            0o37 => Instruction::Rra,
            0o47 => Instruction::Daa,
            0o57 => Instruction::Cpl,
            0o67 => Instruction::Scf,
            0o77 => Instruction::Ccf,
            0o11 | 0o31 | 0o51 | 0o71 => Instruction::Add(
                Operand::RegisterPair(RegisterPair::HL),
                Operand::RegisterPair(inherent_registerpair_operand(opcode >> 3)),
            ),
            0o10 => Instruction::Ld(
                Operand::IndirectImmediate16(self.fetch_imm16()),
                Operand::RegisterPair(RegisterPair::SP),
            ),
            0o02 | 0o22 => Instruction::Ld(
                Operand::RegisterIndirect(inherent_registerpair_operand(opcode >> 3)),
                Operand::Register(Register::A),
            ),
            0o12 | 0o32 => Instruction::Ld(
                Operand::Register(Register::A),
                Operand::RegisterIndirect(inherent_registerpair_operand(opcode >> 3)),
            ),
            0o42 => Instruction::Ld(
                Operand::Register(Register::IncrementHL),
                Operand::Register(Register::A),
            ),
            0o52 => Instruction::Ld(
                Operand::Register(Register::A),
                Operand::Register(Register::IncrementHL),
            ),
            0o62 => Instruction::Ld(
                Operand::Register(Register::DecrementHL),
                Operand::Register(Register::A),
            ),
            0o72 => Instruction::Ld(
                Operand::Register(Register::A),
                Operand::Register(Register::DecrementHL),
            ),
            0o20 => Instruction::Stop,
            0o30 => Instruction::Jr(Condition::Always, self.fetch_imm8() as i8),
            0o40 | 0o50 | 0o60 | 0o70 => Instruction::Jr(
                inherent_condition_operand((opcode - 0o40) >> 3),
                self.fetch_imm8() as i8,
            ),
            0o03 | 0o23 | 0o43 | 0o63 => Instruction::Inc(Operand::RegisterPair(
                inherent_registerpair_operand(opcode >> 3),
            )),
            0o13 | 0o33 | 0o53 | 0o73 => Instruction::Dec(Operand::RegisterPair(
                inherent_registerpair_operand(opcode >> 3),
            )),
            0o04 | 0o14 | 0o24 | 0o34 | 0o44 | 0o54 | 0o64 | 0o74 => Instruction::Inc(
                Operand::Register(inherent_register_operand((opcode & 0o70) >> 3)),
            ),
            0o05 | 0o15 | 0o25 | 0o35 | 0o45 | 0o55 | 0o65 | 0o75 => Instruction::Dec(
                Operand::Register(inherent_register_operand((opcode & 0o70) >> 3)),
            ),
            0o06 | 0o16 | 0o26 | 0o36 | 0o46 | 0o56 | 0o66 | 0o76 => Instruction::Ld(
                Operand::Register(inherent_register_operand((opcode & 0o70) >> 3)),
                Operand::Immediate8(self.fetch_imm8()),
            ),
            0o166 => Instruction::Halt,
            0o100..=0o177 => Instruction::Ld(
                Operand::Register(inherent_register_operand(opcode >> 3)),
                Operand::Register(inherent_register_operand(opcode)),
            ),
            0o200..=0o207 => Instruction::Add(
                Operand::Register(Register::A),
                Operand::Register(inherent_register_operand(opcode)),
            ),
            0o210..=0o217 => Instruction::Adc(Operand::Register(inherent_register_operand(opcode))),
            0o220..=0o227 => Instruction::Sub(Operand::Register(inherent_register_operand(opcode))),
            0o230..=0o237 => Instruction::Sbc(Operand::Register(inherent_register_operand(opcode))),
            0o240..=0o247 => Instruction::And(Operand::Register(inherent_register_operand(opcode))),
            0o250..=0o257 => Instruction::Xor(Operand::Register(inherent_register_operand(opcode))),
            0o260..=0o267 => Instruction::Or(Operand::Register(inherent_register_operand(opcode))),
            0o270..=0o277 => Instruction::Cp(Operand::Register(inherent_register_operand(opcode))),
            0o300 | 0o310 | 0o320 | 0o330 => {
                Instruction::Ret(inherent_condition_operand(opcode >> 3))
            }
            0o301 | 0o321 | 0o341 => {
                Instruction::Pop(inherent_registerpair_operand((opcode & 0o70) >> 3))
            }
            0o305 | 0o325 | 0o345 => {
                Instruction::Push(inherent_registerpair_operand((opcode & 0o70) >> 3))
            }
            0o361 => Instruction::Pop(RegisterPair::AF),
            0o365 => Instruction::Push(RegisterPair::AF),
            0o311 => Instruction::Ret(Condition::Always),
            0o303 => Instruction::Jp(Condition::Always, Operand::Immediate16(self.fetch_imm16())),
            0o351 => Instruction::Jp(Condition::Always, Operand::RegisterPair(RegisterPair::HL)),
            0o302 | 0o312 | 0o322 | 0o332 => Instruction::Jp(
                inherent_condition_operand(opcode >> 3),
                Operand::Immediate16(self.fetch_imm16()),
            ),
            0o304 | 0o314 | 0o324 | 0o334 => {
                Instruction::Call(inherent_condition_operand(opcode >> 3), self.fetch_imm16())
            }
            0o315 => Instruction::Call(Condition::Always, self.fetch_imm16()),
            0o313 => {
                let opcode = self.fetch_imm8();
                match opcode {
                    0o00..=0o07 => Instruction::Rlc(inherent_register_operand(opcode)),
                    0o10..=0o17 => Instruction::Rrc(inherent_register_operand(opcode)),
                    0o20..=0o27 => Instruction::Rl(inherent_register_operand(opcode)),
                    0o30..=0o37 => Instruction::Rr(inherent_register_operand(opcode)),
                    0o40..=0o47 => Instruction::Sla(inherent_register_operand(opcode)),
                    0o50..=0o57 => Instruction::Sra(inherent_register_operand(opcode)),
                    0o60..=0o67 => Instruction::Swap(inherent_register_operand(opcode)),
                    0o70..=0o77 => Instruction::Srl(inherent_register_operand(opcode)),
                    0o100..=0o177 => {
                        Instruction::Bit((opcode - 0o100) >> 3, inherent_register_operand(opcode))
                    }
                    0o200..=0o277 => {
                        Instruction::Res((opcode - 0o200) >> 3, inherent_register_operand(opcode))
                    }
                    0o300..=0o377 => {
                        Instruction::Set((opcode - 0o300) >> 3, inherent_register_operand(opcode))
                    }
                }
            }
            0o306 => Instruction::Add(
                Operand::Register(Register::A),
                Operand::Immediate8(self.fetch_imm8()),
            ),
            0o316 => Instruction::Adc(Operand::Immediate8(self.fetch_imm8())),
            0o326 => Instruction::Sub(Operand::Immediate8(self.fetch_imm8())),
            0o331 => Instruction::Reti,
            0o336 => Instruction::Sbc(Operand::Immediate8(self.fetch_imm8())),
            0o340 => Instruction::Ld(
                Operand::IndirectImmediate8(self.fetch_imm8()),
                Operand::Register(Register::A),
            ),
            0o342 => Instruction::Ld(
                Operand::Register(Register::IndirectC),
                Operand::Register(Register::A),
            ),
            0o346 => Instruction::And(Operand::Immediate8(self.fetch_imm8())),
            0o350 => Instruction::Add(
                Operand::RegisterPair(RegisterPair::SP),
                Operand::Immediate8(self.fetch_imm8()),
            ),
            0o352 => Instruction::Ld(
                Operand::IndirectImmediate16(self.fetch_imm16()),
                Operand::Register(Register::A),
            ),
            0o356 => Instruction::Xor(Operand::Immediate8(self.fetch_imm8())),
            0o360 => Instruction::Ld(
                Operand::Register(Register::A),
                Operand::IndirectImmediate8(self.fetch_imm8()),
            ),
            0o362 => Instruction::Ld(
                Operand::Register(Register::A),
                Operand::Register(Register::IndirectC),
            ),
            0o363 => Instruction::Di,
            0o366 => Instruction::Or(Operand::Immediate8(self.fetch_imm8())),
            0o370 => Instruction::Ld(
                Operand::RegisterPair(RegisterPair::HL),
                Operand::StackOffset(self.fetch_imm8() as i8),
            ),
            0o371 => Instruction::Ld(
                Operand::RegisterPair(RegisterPair::SP),
                Operand::RegisterPair(RegisterPair::HL),
            ),
            0o372 => Instruction::Ld(
                Operand::Register(Register::A),
                Operand::IndirectImmediate16(self.fetch_imm16()),
            ),
            0o373 => Instruction::Ei,
            0o376 => Instruction::Cp(Operand::Immediate8(self.fetch_imm8())),
            0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => {
                Instruction::Rst(((opcode & 0o70) >> 3) * 16)
            }
            _ => {
                panic!(
                    "Unhandled opcode 0x{:02X} at 0x{:04X}",
                    opcode, self.registers.pc
                );
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Nop => (),
            Instruction::Ld(target, source) => match (target, source) {
                (Operand::Register(Register::IndirectHL), Operand::Register(source)) => {
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        self.registers[&source],
                    );
                }
                (Operand::Register(Register::IndirectHL), Operand::Immediate8(value)) => self
                    .bus
                    .write_byte(self.get_register_pair(&RegisterPair::HL), value),
                (Operand::Register(Register::DecrementHL), Operand::Register(source)) => {
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        self.registers[&source],
                    );
                    self.set_register_pair(
                        &RegisterPair::HL,
                        self.get_register_pair(&RegisterPair::HL).wrapping_sub(1),
                    );
                }
                (Operand::Register(Register::IncrementHL), Operand::Register(source)) => {
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        self.registers[&source],
                    );
                    self.set_register_pair(
                        &RegisterPair::HL,
                        self.get_register_pair(&RegisterPair::HL).wrapping_add(1),
                    );
                }
                (Operand::Register(source), Operand::Register(Register::IncrementHL)) => {
                    self.registers[&source] = self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL));
                    self.set_register_pair(
                        &RegisterPair::HL,
                        self.get_register_pair(&RegisterPair::HL).wrapping_add(1),
                    );
                }
                (Operand::Register(Register::IndirectC), Operand::Register(Register::A)) => {
                    self.bus.write_byte(
                        0xFF00 + u16::from(self.registers[&Register::C]),
                        self.registers.a,
                    );
                }
                (Operand::Register(Register::A), Operand::Register(Register::IndirectC)) => {
                    self.registers.a = self
                        .bus
                        .read_byte(0xFF00 + u16::from(self.registers[&Register::C]));
                }
                (Operand::RegisterIndirect(rp), Operand::Register(source)) => self
                    .bus
                    .write_byte(self.get_register_pair(&rp), self.registers[&source]),
                (Operand::Register(source), Operand::RegisterIndirect(rp)) => {
                    self.registers[&source] = self.bus.read_byte(self.get_register_pair(&rp));
                }
                (Operand::Register(source), Operand::Register(Register::IndirectHL)) => {
                    self.registers[&source] = self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL));
                }
                (Operand::IndirectImmediate8(address), Operand::Register(source)) => self
                    .bus
                    .write_byte(0xFF00 + u16::from(address), self.registers[&source]),
                (Operand::Register(source), Operand::IndirectImmediate8(address)) => {
                    self.registers[&source] = self.bus.read_byte(0xFF00 + u16::from(address));
                }
                (Operand::IndirectImmediate16(address), Operand::Register(source)) => {
                    self.bus.write_byte(address, self.registers[&source]);
                }
                (Operand::IndirectImmediate16(address), Operand::RegisterPair(rp)) => {
                    self.bus.write_word(address, self.get_register_pair(&rp));
                }
                (Operand::Register(source), Operand::IndirectImmediate16(address)) => {
                    self.registers[&source] = self.bus.read_byte(address);
                }
                (Operand::Register(target), Operand::Register(Register::DecrementHL)) => {
                    let value = self.get_register_pair(&RegisterPair::HL);
                    self.registers[&target] = self.bus.read_byte(value);
                    let result = value.overflowing_sub(1);
                    self.set_register_pair(&RegisterPair::HL, result.0);
                }
                (Operand::Register(target), Operand::Register(source)) => {
                    self.registers[&target] = self.registers[&source];
                }
                (Operand::Register(target), Operand::Immediate8(value)) => {
                    self.registers[&target] = value;
                }
                (Operand::RegisterPair(target), Operand::Immediate16(value)) => {
                    self.set_register_pair(&target, value);
                }
                (Operand::RegisterPair(target), Operand::RegisterPair(source)) => {
                    self.set_register_pair(&target, self.get_register_pair(&source));
                }
                (Operand::RegisterPair(_), Operand::StackOffset(value)) => {
                    let result = self
                        .get_register_pair(&RegisterPair::SP)
                        .overflowing_add(value as u16);
                    self.flags.z = false;
                    self.flags.n = false;
                    self.flags.h = (self.get_register_pair(&RegisterPair::SP) & 0x0F)
                        + (value as u16 & 0x0F)
                        > 0x0F;
                    self.flags.c = (self.get_register_pair(&RegisterPair::SP) & 0xFF)
                        + (value as u16 & 0xFF)
                        > 0xFF;
                    self.set_register_pair(&RegisterPair::HL, result.0);
                }
                _ => panic!("Illegal operand for LD"),
            },
            Instruction::Add(target, source) => match target {
                Operand::Register(Register::A) => {
                    let value = match source {
                        Operand::Register(Register::IndirectHL) => self
                            .bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL)),
                        Operand::Immediate8(value) => value,
                        _ => match source {
                            Operand::Register(reg) => self.registers[&reg],
                            _ => panic!("Illegal operand for ADD"),
                        },
                    };
                    let result = self.registers.a.overflowing_add(value);
                    self.flags.z = result.0 == 0;
                    self.flags.n = false;
                    self.flags.h = (self.registers.a & 0x0F) + (value & 0x0F) > 0x0F;
                    self.flags.c = result.1;
                    self.registers.a = result.0;
                }
                Operand::RegisterPair(rp) => match source {
                    Operand::RegisterPair(source) => {
                        let result = self
                            .get_register_pair(&rp)
                            .overflowing_add(self.get_register_pair(&source));
                        self.flags.n = false;
                        self.flags.h = (self.get_register_pair(&rp) & 0x0FFF)
                            + (self.get_register_pair(&source) & 0x0FFF)
                            > 0x0FFF;
                        self.flags.c = result.1;
                        self.set_register_pair(&rp, result.0);
                    }
                    Operand::Immediate8(value) => {
                        let result = self
                            .get_register_pair(&rp)
                            .overflowing_add((value as i8) as u16);
                        self.flags.z = false;
                        self.flags.n = false;
                        self.flags.h =
                            (self.get_register_pair(&rp) & 0x0F) + (u16::from(value) & 0x0F) > 0x0F;
                        self.flags.c =
                            (self.get_register_pair(&rp) & 0xFF) + (u16::from(value) & 0xFF) > 0xFF;
                        self.set_register_pair(&rp, result.0);
                    }
                    _ => panic!("Illegal operand for ADD"),
                },
                _ => panic!("Illegal operand for ADD"),
            },
            Instruction::Adc(source) => {
                let value = match source {
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(reg) => self.registers[&reg],
                    Operand::Immediate8(value) => value,
                    _ => panic!("Illegal operand"),
                };
                let mut result = self.registers.a.overflowing_add(value);
                let carry = result.1;
                result = result.0.overflowing_add(u8::from(self.flags.c));
                self.flags.z = result.0 == 0;
                self.flags.n = false;
                self.flags.h =
                    (self.registers.a & 0x0F) + (value & 0x0F) + u8::from(self.flags.c) > 0x0F;
                self.flags.c = carry || result.1;
                self.registers.a = result.0;
            }
            Instruction::Sub(source) => {
                let value = match source {
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(register) => self.registers[&register],
                    Operand::Immediate8(value) => value,
                    _ => panic!("Illegal operand"),
                };
                let result = self.registers.a.overflowing_sub(value);
                self.flags.z = result.0 == 0;
                self.flags.n = true;
                self.flags.h = (self.registers.a & 0x0F) < (value & 0x0F);
                self.flags.c = result.1;
                self.registers.a = result.0;
            }
            Instruction::Sbc(source) => {
                let value = match source {
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(register) => self.registers[&register],
                    Operand::Immediate8(value) => value,
                    _ => panic!("Illegal operand"),
                };
                let mut result = self.registers.a.overflowing_sub(value);
                let carry = result.1;
                result = result.0.overflowing_sub(u8::from(self.flags.c));
                self.flags.z = result.0 == 0;
                self.flags.n = true;
                self.flags.h = (self.registers.a & 0x0F) < (value & 0x0F) + u8::from(self.flags.c);
                self.flags.c = carry || result.1;
                self.registers.a = result.0;
            }
            Instruction::Xor(operand) => {
                self.registers.a ^= match operand {
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(register) => self.registers[&register],
                    Operand::Immediate8(value) => value,
                    _ => panic!("Illegal operand"),
                };
                self.flags.z = self.registers.a == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = false;
            }
            Instruction::And(operand) => {
                self.registers.a &= match operand {
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(register) => self.registers[&register],
                    Operand::Immediate8(value) => value,
                    _ => panic!("Illegal operand"),
                };
                self.flags.z = self.registers.a == 0;
                self.flags.n = false;
                self.flags.h = true;
                self.flags.c = false;
            }
            Instruction::Or(operand) => {
                self.registers.a |= match operand {
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(register) => self.registers[&register],
                    Operand::Immediate8(value) => value,
                    _ => panic!("Illegal operand"),
                };
                self.flags.z = self.registers.a == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = false;
            }
            Instruction::Di => {
                self.ime = false;
            }
            Instruction::Ei => {
                self.ime = true; // TODO delay
            }
            Instruction::Bit(bit, register) => {
                let value = match register {
                    Register::IndirectHL => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    _ => self.registers[&register],
                } & (1 << bit);
                self.flags.z = value == 0;
                self.flags.n = false;
                self.flags.h = true;
            }
            Instruction::Set(bit, register) => match register {
                Register::IndirectHL => self.bus.write_byte(
                    self.get_register_pair(&RegisterPair::HL),
                    self.bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL))
                        | (1 << bit),
                ),
                _ => self.registers[&register] |= 1 << bit,
            },
            Instruction::Res(bit, register) => match register {
                Register::IndirectHL => self.bus.write_byte(
                    self.get_register_pair(&RegisterPair::HL),
                    self.bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL))
                        & !(1 << bit),
                ),
                _ => self.registers[&register] &= !(1 << bit),
            },
            Instruction::Push(rp) => {
                self.push(self.get_register_pair(&rp));
            }
            Instruction::Pop(rp) => {
                let result = self.pop();
                self.set_register_pair(&rp, result);
            }
            Instruction::Rst(address) => {
                self.push(self.registers.pc);
                self.registers.pc = u16::from(address);
            }
            Instruction::Call(condition, address) => {
                if match condition {
                    Condition::Always => true,
                    Condition::Carry => self.flags.c,
                    Condition::NonCarry => !self.flags.c,
                    Condition::Zero => self.flags.z,
                    Condition::NonZero => !self.flags.z,
                } {
                    self.push(self.registers.pc);
                    self.registers.pc = address;
                }
            }
            Instruction::Jp(condition, operand) => {
                if match condition {
                    Condition::Always => true,
                    Condition::Carry => self.flags.c,
                    Condition::NonCarry => !self.flags.c,
                    Condition::Zero => self.flags.z,
                    Condition::NonZero => !self.flags.z,
                } {
                    match operand {
                        Operand::RegisterPair(RegisterPair::HL) => {
                            self.registers.pc = self.get_register_pair(&RegisterPair::HL);
                        }
                        Operand::Immediate16(address) => self.registers.pc = address,
                        _ => panic!("Illegal operand"),
                    }
                }
            }
            Instruction::Jr(condition, offset) => {
                if match condition {
                    Condition::Always => true,
                    Condition::Carry => self.flags.c,
                    Condition::NonCarry => !self.flags.c,
                    Condition::Zero => self.flags.z,
                    Condition::NonZero => !self.flags.z,
                } {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                }
            }
            Instruction::Ret(condition) => {
                if match condition {
                    Condition::Always => true,
                    Condition::Carry => self.flags.c,
                    Condition::NonCarry => !self.flags.c,
                    Condition::Zero => self.flags.z,
                    Condition::NonZero => !self.flags.z,
                } {
                    self.registers.pc = self.pop();
                }
            }
            Instruction::Reti => {
                self.registers.pc = self.pop();
                self.ime = true;
            }
            Instruction::Inc(operand) => {
                match operand {
                    Operand::RegisterPair(rp) => {
                        self.set_register_pair(&rp, self.get_register_pair(&rp).wrapping_add(1));
                    }
                    Operand::Register(register) => {
                        let (value, result) = if let Register::IndirectHL = register {
                            let value = self
                                .bus
                                .read_byte(self.get_register_pair(&RegisterPair::HL));
                            let result = value.wrapping_add(1);
                            self.bus
                                .write_byte(self.get_register_pair(&RegisterPair::HL), result);
                            (value, result)
                        } else {
                            let value = self.registers[&register];
                            let result = value.wrapping_add(1);
                            self.registers[&register] = result;
                            (value, result)
                        };
                        self.flags.z = result == 0;
                        self.flags.n = false;
                        self.flags.h = (value & 0x0F) + 1 > 0x0F; // TODO
                    }
                    _ => panic!("Illegal operand"),
                }
            }
            Instruction::Dec(operand) => {
                match operand {
                    Operand::RegisterPair(rp) => {
                        self.set_register_pair(&rp, self.get_register_pair(&rp).wrapping_sub(1));
                    }
                    Operand::Register(register) => {
                        let result = if let Register::IndirectHL = register {
                            let value = self
                                .bus
                                .read_byte(self.get_register_pair(&RegisterPair::HL));
                            let result = value.wrapping_sub(1);
                            self.bus
                                .write_byte(self.get_register_pair(&RegisterPair::HL), result);
                            result
                        } else {
                            let value = self.registers[&register];
                            let result = value.wrapping_sub(1);
                            self.registers[&register] = result;
                            result
                        };
                        self.flags.z = result == 0;
                        self.flags.n = true;
                        self.flags.h = (result & 0x0F) + 1 > 0x0F; // TODO
                    }
                    _ => panic!("Illegal operand"),
                }
            }
            Instruction::Rl(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            << 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x80
                            != 0,
                    );
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        result.0 | u8::from(self.flags.c),
                    );
                    result
                } else {
                    let result = (
                        self.registers[&register] << 1,
                        self.registers[&register] & 0x80 != 0,
                    );
                    self.registers[&register] = result.0 | u8::from(self.flags.c);
                    result
                };
                self.flags.z = result.0 | u8::from(self.flags.c) == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rr(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            >> 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x01
                            != 0,
                    );
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        result.0 | if self.flags.c { 0x80 } else { 0 },
                    );
                    result
                } else {
                    let result = (
                        self.registers[&register] >> 1,
                        self.registers[&register] & 0x01 != 0,
                    );
                    self.registers[&register] = result.0 | if self.flags.c { 0x80 } else { 0 };
                    result
                };
                self.flags.z = result.0 | if self.flags.c { 0x80 } else { 0 } == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rla => {
                let result = (
                    self.registers[&Register::A] << 1,
                    self.registers[&Register::A] & 0x80 != 0,
                );
                self.registers[&Register::A] = result.0 | u8::from(self.flags.c);
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rra => {
                let result = (
                    self.registers[&Register::A] >> 1,
                    self.registers[&Register::A] & 0x01 != 0,
                );
                self.registers[&Register::A] = result.0 | if self.flags.c { 0x80 } else { 0 };
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rlca => {
                let result = (
                    self.registers[&Register::A] << 1,
                    self.registers[&Register::A] & 0x80 != 0,
                );
                self.registers[&Register::A] = result.0 | u8::from(result.1);
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rrca => {
                let result = (
                    self.registers[&Register::A] >> 1,
                    self.registers[&Register::A] & 0x01 != 0,
                );
                self.registers[&Register::A] = result.0 | if result.1 { 0x80 } else { 0 };
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rlc(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            << 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x80
                            != 0,
                    );
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        result.0 | u8::from(result.1),
                    );
                    result
                } else {
                    let mut result: (u8, bool) = (
                        self.registers[&register] << 1,
                        self.registers[&register] & 0x80 != 0,
                    );
                    result = (result.0 | u8::from(result.1), result.1);
                    self.registers[&register] = result.0;
                    result
                };
                self.flags.z = result.0 == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Rrc(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            >> 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x01
                            != 0,
                    );
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        result.0 | if result.1 { 0x80 } else { 0 },
                    );
                    result
                } else {
                    let mut result = (
                        self.registers[&register] >> 1,
                        self.registers[&register] & 0x01 != 0,
                    );
                    result = (result.0 | if result.1 { 0x80 } else { 0 }, result.1);
                    self.registers[&register] = result.0;
                    result
                };
                self.flags.z = result.0 == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Scf => {
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = true;
            }
            Instruction::Ccf => {
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = !self.flags.c;
            }
            Instruction::Sla(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            << 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x80
                            != 0,
                    );
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        result.0 | ((result.0 >> 1) & 1),
                    );
                    result
                } else {
                    let result = (
                        self.registers[&register] << 1,
                        self.registers[&register] & 0x80 != 0,
                    );
                    self.registers[&register] = result.0 | ((result.0 << 1) & 1);
                    result
                };
                self.flags.z = result.0 == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Sra(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            >> 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x01
                            != 0,
                    );
                    self.bus.write_byte(
                        self.get_register_pair(&RegisterPair::HL),
                        result.0 | ((result.0 >> 1) & 1),
                    );
                    result
                } else {
                    let mut result = (
                        self.registers[&register] >> 1,
                        self.registers[&register] & 0x01 != 0,
                    );
                    result = (result.0 | ((self.registers[&register]) & 0x80), result.1);
                    self.registers[&register] = result.0;
                    result
                };
                self.flags.z = result.0 == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Srl(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = (
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            >> 1,
                        self.bus
                            .read_byte(self.get_register_pair(&RegisterPair::HL))
                            & 0x01
                            != 0,
                    );
                    self.bus
                        .write_byte(self.get_register_pair(&RegisterPair::HL), result.0);
                    result
                } else {
                    let result = (
                        self.registers[&register] >> 1,
                        self.registers[&register] & 0x01 != 0,
                    );
                    self.registers[&register] = result.0;
                    result
                };
                self.flags.z = result.0 == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = result.1;
            }
            Instruction::Swap(register) => {
                let result = if let Register::IndirectHL = register {
                    let result = self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL))
                        .rotate_right(4);
                    self.bus
                        .write_byte(self.get_register_pair(&RegisterPair::HL), result);
                    result
                } else {
                    let result = self.registers[&register].rotate_right(4);
                    self.registers[&register] = result;
                    result
                };
                self.flags.z = result == 0;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = false;
            }
            Instruction::Cp(operand) => {
                let value = match operand {
                    Operand::Immediate8(value) => value,
                    Operand::Register(Register::IndirectHL) => self
                        .bus
                        .read_byte(self.get_register_pair(&RegisterPair::HL)),
                    Operand::Register(reg) => self.registers[&reg],
                    _ => panic!("Unhandled operand"),
                };
                let result = self.registers.a.overflowing_sub(value);
                self.flags.z = result.0 == 0;
                self.flags.n = true;
                self.flags.h = (self.registers.a & 0x0F) < (value & 0x0F);
                self.flags.c = result.1;
            }
            Instruction::Cpl => {
                self.registers.a = !self.registers.a;
                self.flags.n = true;
                self.flags.h = true;
            }
            _ => panic!("Unhandled instruction {instruction:?}"),
        }
    }
}
