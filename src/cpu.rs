use crate::Bus;

pub struct CPU {
  pub bus: Bus,
  pub registers: Registers,
  pub flags: Flags,
  pub ime: bool,
}

pub struct Flags {
  pub z: bool,
  pub c: bool,
  pub n: bool,
  pub h: bool
}

pub struct Registers {
  pub a: u8,
  pub b: u8,
  pub c: u8,
  pub d: u8,
  pub e: u8,
  pub h: u8,
  pub l: u8,
  pub pc: u16,
  pub sp: u16
}

use std::ops::{Index,IndexMut};

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
            _ => panic!("Unknown register")
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
            _ => panic!("Unknown register")
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
  PC,
  AF
}

fn inherent_condition_operand(opcode: &u8) -> Condition {
  match opcode & 0o03 {
    0 => Condition::NonZero,
    1 => Condition::Zero,
    2 => Condition::NonCarry,
    3 => Condition::Carry,
    _ => panic!("This should never happen")
  }
}

fn inherent_register_operand(opcode: &u8) -> Register {
  match opcode & 0o07 {
    0 => Register::B,
    1 => Register::C,
    2 => Register::D,
    3 => Register::E,
    4 => Register::H,
    5 => Register::L,
    6 => Register::IndirectHL,
    7 => Register::A,
    _ => panic!("This should never happen")
  }
}

fn inherent_registerpair_operand(opcode: &u8) -> RegisterPair {
  match opcode & 0o07 {
    0 | 1 => RegisterPair::BC,
    2 | 3 => RegisterPair::DE,
    4 | 5 => RegisterPair::HL,
    6 | 7 => RegisterPair::SP,
    _ => panic!("This should never happen")
  }
}

#[derive(Debug)]
pub enum Condition {
  Always, Zero, NonZero, Carry, NonCarry
}

#[derive(Debug)]
pub enum Instruction {
  LD(Operand, Operand),
  XOR(Operand),
  AND(Operand),
  ADD(Operand, Operand),
  ADC(Operand),
  SUB(Operand),
  SBC(Operand),
  OR(Operand),
  CP(Operand),
  INC(Operand),
  DEC(Operand),
  RLC(Register),
  RRC(Register),
  RL(Register),
  RLA,
  RLCA,
  RR(Register),
  RRA,
  RRCA,
  SLA(Register),
  SRA(Register),
  SWAP(Register),
  SRL(Register),
  BIT(u8, Register),
  RES(u8, Register),
  SET(u8, Register),
  RST(u8),
  RET(Condition),
  RETI,
  JP(Condition, Operand),
  JR(Condition, i8),
  CALL(Condition, u16),
  STOP,
  NOP,
  HALT,
  EI,
  DI,
  PUSH(RegisterPair),
  POP(RegisterPair),
  DAA,
  CPL,
  SCF,
  CCF,
}

#[derive(Debug)]
pub enum Operand {
  Immediate8(u8),
  IndirectImmediate8(u8),
  Immediate16(u16),
  IndirectImmediate16(u16),
  Register(Register),
  RegisterPair(RegisterPair),
  RegisterIndirect(RegisterPair)
}

impl CPU {
  pub fn get_register_pair(&self, rp: &RegisterPair) -> u16 {
    match rp {
      RegisterPair::BC => ((self.registers.b as u16) << 8) | self.registers.c as u16,
      RegisterPair::DE => ((self.registers.d as u16) << 8) | self.registers.e as u16,
      RegisterPair::HL => ((self.registers.h as u16) << 8) | self.registers.l as u16,
      RegisterPair::AF => ((self.registers.a as u16) << 8) | if self.flags.z { 0x80 } else { 0 } | if self.flags.n { 0x40 } else { 0 } | if self.flags.h { 0x20 } else { 0 } | if self.flags.c { 0x10 } else { 0 },
      RegisterPair::SP => self.registers.sp,
      _ => panic!("Illegal register pair")
    }
  }

  fn set_register_pair(&mut self, rp: &RegisterPair, value: u16) {
    match rp {
      RegisterPair::AF => {
        self.registers.a = (value >> 8) as u8;
        self.flags.z = value & 0x80 == 0x80;
        self.flags.n = value & 0x40 == 0x40;
        self.flags.h = value & 0x20 == 0x20;
        self.flags.c = value & 0x10 == 0x10;
      },
      RegisterPair::BC => {
        self.registers.b = (value >> 8) as u8;
        self.registers.c = value as u8;
      },
      RegisterPair::DE => {
        self.registers.d = (value >> 8) as u8;
        self.registers.e = value as u8;
      },
      RegisterPair::HL => {
        self.registers.h = (value >> 8) as u8;
        self.registers.l = value as u8;
      },
      RegisterPair::SP => self.registers.sp = value,
      _ => panic!("Illegal register pair")
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
    self.registers.sp = self.registers.sp.wrapping_sub(1);
    self.bus.write_byte(self.registers.sp, (value >> 8) as u8);
    self.registers.sp = self.registers.sp.wrapping_sub(1);
    self.bus.write_byte(self.registers.sp, (value & 0xFF) as u8);
  }
  
  fn pop(&mut self) -> u16 {
    let mut result = self.bus.read_byte(self.registers.sp) as u16;
    self.registers.sp = self.registers.sp.wrapping_add(1);
    result |= (self.bus.read_byte(self.registers.sp) as u16) << 8;
    self.registers.sp = self.registers.sp.wrapping_add(1);
    result
  }

  pub fn decode(&mut self, opcode: u8) -> Instruction {
    #[allow(clippy::match_overlapping_arm)]
    match opcode {
      0o00 => Instruction::NOP,
      0o01 | 0o21 | 0o41 | 0o61 => Instruction::LD(Operand::RegisterPair(inherent_registerpair_operand(&(opcode >> 3))), Operand::Immediate16(self.fetch_imm16())),
      0o07 => Instruction::RLCA,
      0o17 => Instruction::RRCA,
      0o27 => Instruction::RLA,
      0o37 => Instruction::RRA,
      0o47 => Instruction::DAA,
      0o57 => Instruction::CPL,
      0o67 => Instruction::SCF,
      0o77 => Instruction::CCF,
      0o11 | 0o31 | 0o51 | 0o71 => Instruction::ADD(Operand::RegisterPair(RegisterPair::HL), Operand::RegisterPair(inherent_registerpair_operand(&(opcode >> 3)))),
      0o02 | 0o22 => Instruction::LD(Operand::RegisterIndirect(inherent_registerpair_operand(&(opcode >> 3))), Operand::Register(Register::A)),
      0o12 | 0o32 => Instruction::LD(Operand::Register(Register::A), Operand::RegisterIndirect(inherent_registerpair_operand(&(opcode >> 3)))),
      0o42 => Instruction::LD(Operand::Register(Register::IncrementHL), Operand::Register(Register::A)),
      0o52 => Instruction::LD(Operand::Register(Register::A), Operand::Register(Register::IncrementHL)),
      0o62 => Instruction::LD(Operand::Register(Register::DecrementHL), Operand::Register(Register::A)),
      0o72 => Instruction::LD(Operand::Register(Register::A), Operand::Register(Register::DecrementHL)),
      0o20 => Instruction::STOP,
      0o30 => Instruction::JR(Condition::Always, self.fetch_imm8() as i8),
      0o40 | 0o50 | 0o60 | 0o70 => Instruction::JR(inherent_condition_operand(&((opcode - 0o40) >> 3)), self.fetch_imm8() as i8),
      0o03 | 0o23 | 0o43 | 0o63 => Instruction::INC(Operand::RegisterPair(inherent_registerpair_operand(&(opcode >> 3)))),
      0o13 | 0o33 | 0o53 | 0o73 => Instruction::DEC(Operand::RegisterPair(inherent_registerpair_operand(&(opcode >> 3)))),
      0o04 | 0o14 | 0o24 | 0o34 | 0o44 | 0o54 | 0o64 | 0o74 => Instruction::INC(Operand::Register(inherent_register_operand(&((opcode & 0o70) >> 3)))),
      0o05 | 0o15 | 0o25 | 0o35 | 0o45 | 0o55 | 0o65 | 0o75 => Instruction::DEC(Operand::Register(inherent_register_operand(&((opcode & 0o70) >> 3)))),
      0o06 | 0o16 | 0o26 | 0o36 | 0o46 | 0o56 | 0o66 | 0o76 => Instruction::LD(Operand::Register(inherent_register_operand(&((opcode & 0o70) >> 3))), Operand::Immediate8(self.fetch_imm8())),
      0o166 => Instruction::HALT,
      0o100..=0o177 => Instruction::LD(Operand::Register(inherent_register_operand(&(opcode >> 3))), Operand::Register(inherent_register_operand(&opcode))),
      0o200..=0o207 => Instruction::ADD(Operand::Register(Register::A), Operand::Register(inherent_register_operand(&opcode))),
      0o210..=0o217 => Instruction::ADC(Operand::Register(inherent_register_operand(&opcode))),
      0o220..=0o227 => Instruction::SUB(Operand::Register(inherent_register_operand(&opcode))),
      0o230..=0o237 => Instruction::SBC(Operand::Register(inherent_register_operand(&opcode))),
      0o240..=0o247 => Instruction::AND(Operand::Register(inherent_register_operand(&opcode))),
      0o250..=0o257 => Instruction::XOR(Operand::Register(inherent_register_operand(&opcode))),
      0o260..=0o267 => Instruction::OR(Operand::Register(inherent_register_operand(&opcode))),
      0o270..=0o277 => Instruction::CP(Operand::Register(inherent_register_operand(&opcode))),
      0o300 | 0o310 | 0o320 | 0o330 => Instruction::RET(inherent_condition_operand(&(opcode >> 3))),
      0o301 | 0o321 | 0o341 => Instruction::POP(inherent_registerpair_operand(&((opcode & 0o70) >> 3))),
      0o305 | 0o325 | 0o345 => Instruction::PUSH(inherent_registerpair_operand(&((opcode & 0o70) >> 3))),
      0o361 => Instruction::POP(RegisterPair::AF),
      0o365 => Instruction::PUSH(RegisterPair::AF),
      0o311 => Instruction::RET(Condition::Always),
      0o303 => Instruction::JP(Condition::Always, Operand::Immediate16(self.fetch_imm16())),
      0o351 => Instruction::JP(Condition::Always, Operand::RegisterPair(RegisterPair::HL)),
      0o302 | 0o312 | 0o322 | 0o332 => Instruction::JP(inherent_condition_operand(&(opcode >> 3)), Operand::Immediate16(self.fetch_imm16())),
      0o304 | 0o314 | 0o324 | 0o334 => Instruction::CALL(inherent_condition_operand(&(opcode >> 3)), self.fetch_imm16()),
      0o315 => Instruction::CALL(Condition::Always, self.fetch_imm16()),
      0o313 => {
        let opcode = self.fetch_imm8();
        match opcode {
          0o00..=0o07 => Instruction::RLC(inherent_register_operand(&opcode)),
          0o10..=0o17 => Instruction::RRC(inherent_register_operand(&opcode)),
          0o20..=0o27 => Instruction::RL(inherent_register_operand(&opcode)),
          0o30..=0o37 => Instruction::RR(inherent_register_operand(&opcode)),
          0o40..=0o47 => Instruction::SLA(inherent_register_operand(&opcode)),
          0o50..=0o57 => Instruction::SRA(inherent_register_operand(&opcode)),
          0o60..=0o67 => Instruction::SWAP(inherent_register_operand(&opcode)),
          0o70..=0o77 => Instruction::SRL(inherent_register_operand(&opcode)),
          0o100..=0o177 => Instruction::BIT((opcode - 0o100) >> 3, inherent_register_operand(&opcode)),
          0o200..=0o277 => Instruction::RES((opcode - 0o100) >> 3, inherent_register_operand(&opcode)),
          0o300..=0o377 => Instruction::SET((opcode - 0o100) >> 3, inherent_register_operand(&opcode)),
        }
      },
      0o306 => Instruction::ADD(Operand::Register(Register::A), Operand::Immediate8(self.fetch_imm8())),
      0o316 => Instruction::ADC(Operand::Immediate8(self.fetch_imm8())),
      0o326 => Instruction::SUB(Operand::Immediate8(self.fetch_imm8())),
      0o336 => Instruction::SBC(Operand::Immediate8(self.fetch_imm8())),
      0o340 => Instruction::LD(Operand::IndirectImmediate8(self.fetch_imm8()), Operand::Register(Register::A)),
      0o342 => Instruction::LD(Operand::Register(Register::IndirectC), Operand::Register(Register::A)),
      0o346 => Instruction::AND(Operand::Immediate8(self.fetch_imm8())),
      0o352 => Instruction::LD(Operand::IndirectImmediate16(self.fetch_imm16()), Operand::Register(Register::A)),
      0o356 => Instruction::XOR(Operand::Immediate8(self.fetch_imm8())),
      0o360 => Instruction::LD(Operand::Register(Register::A), Operand::IndirectImmediate8(self.fetch_imm8())),
      0o363 => Instruction::DI,
      0o366 => Instruction::OR(Operand::Immediate8(self.fetch_imm8())),
      0o372 => Instruction::LD(Operand::Register(Register::A), Operand::IndirectImmediate16(self.fetch_imm16())),
      0o373 => Instruction::EI,
      0o376 => Instruction::CP(Operand::Immediate8(self.fetch_imm8())),
      0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => Instruction::RST(((opcode & 0o70) >> 3) * 16),
      _ => {
        panic!("Unhandled opcode 0x{:X}", opcode);
      }
    }
  }

  pub fn execute(&mut self, instruction: Instruction) {
    match instruction {
      Instruction::NOP => (),
      Instruction::LD(target, source) => {
        match (target, source) {
          (Operand::Register(Register::IndirectHL), Operand::Register(source)) => self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), self.registers[&source]),
          (Operand::Register(Register::IndirectHL), Operand::Immediate8(value)) => self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), value),
          (Operand::Register(Register::DecrementHL), Operand::Register(source)) => {
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), self.registers[&source]);
            self.set_register_pair(&RegisterPair::HL, self.get_register_pair(&RegisterPair::HL).wrapping_sub(1));
          },
          (Operand::Register(Register::IncrementHL), Operand::Register(source)) => {
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), self.registers[&source]);
            self.set_register_pair(&RegisterPair::HL, self.get_register_pair(&RegisterPair::HL).wrapping_add(1));
          },
          (Operand::Register(source), Operand::Register(Register::IncrementHL)) => {
            self.registers[&source] = self.bus.read_byte(self.get_register_pair(&RegisterPair::HL));
            self.set_register_pair(&RegisterPair::HL, self.get_register_pair(&RegisterPair::HL).wrapping_add(1));
          },
          (Operand::Register(Register::IndirectC), Operand::Register(source)) => self.bus.write_byte(0xFF00 + self.registers[&Register::C] as u16, self.registers[&source]),
          (Operand::RegisterIndirect(rp), Operand::Register(source)) => self.bus.write_byte(self.get_register_pair(&rp), self.registers[&source]),
          (Operand::Register(source), Operand::RegisterIndirect(rp)) => self.registers[&source] = self.bus.read_byte(self.get_register_pair(&rp)),
          (Operand::Register(source), Operand::Register(Register::IndirectHL)) => self.registers[&source] = self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)),
          (Operand::IndirectImmediate8(address), Operand::Register(source)) => self.bus.write_byte(0xFF00 + address as u16, self.registers[&source]),
          (Operand::Register(source), Operand::IndirectImmediate8(address)) => self.registers[&source] = self.bus.read_byte(0xFF00 + address as u16),
          (Operand::IndirectImmediate16(address), Operand::Register(source)) => self.bus.write_byte(address, self.registers[&source]),
          (Operand::Register(source), Operand::IndirectImmediate16(address)) => self.registers[&source] = self.bus.read_byte(address),
          (Operand::Register(target), Operand::Register(source)) => self.registers[&target] = self.registers[&source],
          (Operand::Register(target), Operand::Immediate8(value)) => self.registers[&target] = value,
          (Operand::RegisterPair(target), Operand::Immediate16(value)) => self.set_register_pair(&target, value),
          _ => panic!("Illegal operand")
        }
      },
      Instruction::ADD(target, source) => {
        match target {
          Operand::Register(Register::A) => {
            let result = match source {
              Operand::Register(Register::IndirectHL) => self.registers.a.overflowing_add(self.bus.read_byte(self.get_register_pair(&RegisterPair::HL))),
              Operand::Immediate8(value) => self.registers.a.overflowing_add(value),
              _ => match source {
                Operand::Register(reg) => self.registers.a.overflowing_add(self.registers[&reg]),
                _ => panic!("Illegal operand")
              }
            };
            self.flags.z = result.0 == 0;
            self.flags.n = false;
            self.flags.h = false;
            self.flags.c = result.1;
            self.registers.a = result.0;
          },
          Operand::RegisterPair(rp) => {
            if let Operand::RegisterPair(source) = source {
              let result = self.get_register_pair(&rp).overflowing_add(self.get_register_pair(&source));
              self.flags.n = false;
              self.flags.h = false; // TODO
              self.flags.c = result.1;
              self.set_register_pair(&rp, result.0);
            };
          },
          _ => panic!("Illegal operand")
        }
      },
      Instruction::ADC(source) => {
        let value = match source {
          Operand::Register(reg) => self.registers[&reg],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal operand")
        };
        let result = self.registers.a.overflowing_add(value + if self.flags.c {1} else {0});
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
        self.registers.a = result.0;
      },
      Instruction::SUB(source) => {
        let value = match source {
          Operand::Register(register) => self.registers[&register],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal operand")
        };
        let result = self.registers.a.overflowing_sub(value);
        self.flags.z = result.0 == 0;
        self.flags.n = true;
        self.flags.h = false;
        self.flags.c = result.1;
        self.registers.a = result.0;
      },
      Instruction::SBC(source) => {
        let value = match source {
          Operand::Register(register) => self.registers[&register],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal operand")
        };
        let result = self.registers.a.overflowing_sub(value + if self.flags.c {1} else {0});
        self.flags.z = result.0 == 0;
        self.flags.n = true;
        self.flags.h = false;
        self.flags.c = result.1;
        self.registers.a = result.0;
      },
      Instruction::XOR(operand) => {
        self.registers.a ^= match operand {
          Operand::Register(Register::IndirectHL) => self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)),
          Operand::Register(register) => self.registers[&register],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal operand")
        };
        self.flags.z = self.registers.a == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
      },
      Instruction::AND(operand) => {
        self.registers.a &= match operand {
          Operand::Register(Register::IndirectHL) => self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)),
          Operand::Register(register) => self.registers[&register],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal operand")
        };
        self.flags.z = self.registers.a == 0;
        self.flags.n = false;
        self.flags.h = true;
        self.flags.c = false;
      },
      Instruction::OR(operand) => {
        self.registers.a |= match operand {
          Operand::Register(Register::IndirectHL) => self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)),
          Operand::Register(register) => self.registers[&register],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal operand")
        };
        self.flags.z = self.registers.a == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
      },
      Instruction::DI => {
        self.ime = false;
      },
      Instruction::EI => {
        self.ime = true; // TODO delay
      },
      Instruction::BIT(bit, register) => {
        self.flags.z = self.registers[&register] & (1 << bit) == 0;
        self.flags.n = false;
        self.flags.h = true;
      },
      Instruction::PUSH(rp) => {
        self.push(self.get_register_pair(&rp));
      },
      Instruction::POP(rp) => {
        let result = self.pop();
        self.set_register_pair(&rp, result);
      },
      Instruction::CALL(condition, address) => {
        if match condition {
          Condition::Always => true,
          Condition::Carry => self.flags.c,
          Condition::NonCarry => !self.flags.c,
          Condition::Zero => self.flags.z,
          Condition::NonZero => !self.flags.z
        } {
          self.push(self.registers.pc);
          self.registers.pc = address;
        }
      },
      Instruction::JP(condition, operand) => {
        if match condition {
          Condition::Always => true,
          Condition::Carry => self.flags.c,
          Condition::NonCarry => !self.flags.c,
          Condition::Zero => self.flags.z,
          Condition::NonZero => !self.flags.z
        } {
          match operand {
            Operand::RegisterPair(RegisterPair::HL) => self.registers.pc = self.get_register_pair(&RegisterPair::HL),
            Operand::Immediate16(address) => self.registers.pc = address,
            _ => panic!("Illegal operand")
          }
        }
      },
      Instruction::JR(condition, offset) => {
        if match condition {
          Condition::Always => true,
          Condition::Carry => self.flags.c,
          Condition::NonCarry => !self.flags.c,
          Condition::Zero => self.flags.z,
          Condition::NonZero => !self.flags.z
        } {
          self.registers.pc = self.registers.pc.wrapping_add(offset as u16)
        }
      },
      Instruction::RET(condition) => {
        if match condition {
          Condition::Always => true,
          Condition::Carry => self.flags.c,
          Condition::NonCarry => !self.flags.c,
          Condition::Zero => self.flags.z,
          Condition::NonZero => !self.flags.z
        } {
          self.registers.pc = self.pop();
        }
      },
      Instruction::INC(operand) => {
        match operand {
          Operand::RegisterPair(rp) => {
            self.set_register_pair(&rp, self.get_register_pair(&rp).wrapping_add(1));
          },
          Operand::Register(register) => {
            let (value, result) = match register {
              Register::IndirectHL => {
                let value = self.bus.read_byte(self.get_register_pair(&RegisterPair::HL));
                let result = value.wrapping_add(1);
                self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result);
                (value, result)
              },
              _ => {
                let value = self.registers[&register];
                let result = value.wrapping_add(1);
                self.registers[&register] = result;
                (value, result)
              }
            };
            self.flags.z = result == 0;
            self.flags.n = false;
            self.flags.h = false; // TODO
          },
          _ => panic!("Illegal operand")
        }
      },
      Instruction::DEC(operand) => {
        match operand {
          Operand::RegisterPair(rp) => {
            self.set_register_pair(&rp, self.get_register_pair(&rp).wrapping_sub(1));
          },
          Operand::Register(register) => {
            let (value, result) = match register {
              Register::IndirectHL => {
                let value = self.bus.read_byte(self.get_register_pair(&RegisterPair::HL));
                let result = value.wrapping_sub(1);
                self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result);
                (value, result)
              },
              _ => {
                let value = self.registers[&register];
                let result = value.wrapping_sub(1);
                self.registers[&register] = result;
                (value, result)
              }
            };
            self.flags.z = result == 0;
            self.flags.n = true;
            self.flags.h = false; // TODO
          },
          _ => panic!("Illegal operand")
        }
      },
      Instruction::RL(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) << 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x80 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0 | if self.flags.c {1} else {0});
            result
          },
          _ => {
            let result = (self.registers[&register] << 1, self.registers[&register] & 0x80 != 0);
            self.registers[&register] = result.0 | if self.flags.c {1} else {0};
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RR(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) >> 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x01 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0 | if self.flags.c {1} else {0});
            result
          },
          _ => {
            let result = (self.registers[&register] >> 1, self.registers[&register] & 0x01 != 0);
            self.registers[&register] = result.0 | if self.flags.c {1} else {0};
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RLA => {
        let result = (self.registers[&Register::A] << 1, self.registers[&Register::A] & 0x80 != 0);
        self.registers[&Register::A] = result.0 | if self.flags.c {1} else {0};
        self.flags.z = false;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RRA => {
        let result = (self.registers[&Register::A] >> 1, self.registers[&Register::A] & 0x01 != 0);
        self.registers[&Register::A] = result.0 | if self.flags.c {1} else {0};
        self.flags.z = false;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RLCA => {
        let result = (self.registers[&Register::A] << 1, self.registers[&Register::A] & 0x80 != 0);
        self.registers[&Register::A] = result.0 | if result.1 {1} else {0};
        self.flags.z = false;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RRCA => {
        let result = (self.registers[&Register::A] >> 1, self.registers[&Register::A] & 0x01 != 0);
        self.registers[&Register::A] = result.0 | if result.1 {1} else {0};
        self.flags.z = false;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RLC(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) << 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x80 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0 | if result.1 {1} else {0});
            result
          },
          _ => {
            let result = (self.registers[&register] << 1, self.registers[&register] & 0x80 != 0);
            self.registers[&register] = result.0 | if result.1 {1} else {0};
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::RRC(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) >> 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x01 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0 | if result.1 {1} else {0});
            result
          },
          _ => {
            let result = (self.registers[&register] >> 1, self.registers[&register] & 0x01 != 0);
            self.registers[&register] = result.0 | if result.1 {1} else {0};
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::SLA(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) << 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x80 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0 | ((result.0 >> 1) & 1));
            result
          },
          _ => {
            let result = (self.registers[&register] << 1, self.registers[&register] & 0x80 != 0);
            self.registers[&register] = result.0 | ((result.0 >> 1) & 1);
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::SRA(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) >> 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x01 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0 | ((result.0 >> 1) & 1));
            result
          },
          _ => {
            let result = (self.registers[&register] >> 1, self.registers[&register] & 0x01 != 0);
            self.registers[&register] = result.0 | ((result.0 >> 1) & 1);
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::SRL(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = (self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) >> 1, self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)) & 0x01 != 0);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result.0);
            result
          },
          _ => {
            let result = (self.registers[&register] >> 1, self.registers[&register] & 0x01 != 0);
            self.registers[&register] = result.0;
            result
          }
        };
        self.flags.z = result.0 == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      Instruction::SWAP(register) => {
        let result = match register {
          Register::IndirectHL => {
            let result = self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)).rotate_right(4);
            self.bus.write_byte(self.get_register_pair(&RegisterPair::HL), result);
            result
          },
          _ => {
            let result = self.registers[&register].rotate_right(4);
            self.registers[&register] = result;
            result
          }
        };
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
      },
      Instruction::CP(operand) => {
        let result = self.registers.a.overflowing_sub(match operand {
          Operand::Immediate8(value) => value,
          Operand::Register(Register::IndirectHL) => self.bus.read_byte(self.get_register_pair(&RegisterPair::HL)),
          Operand::Register(reg) => self.registers[&reg],
          _ => panic!("Unhandled operand")
        });
        self.flags.z = result.0 == 0;
        self.flags.n = true;
        self.flags.h = false;
        self.flags.c = result.1;
      },
      _ => panic!("Unhandled instruction")
    }
  }
}