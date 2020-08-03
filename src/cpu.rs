use crate::Bus;

pub struct CPU {
  pub bus: Bus,
  pub registers: Registers,
  pub flags: Flags
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

impl Index<Register> for Registers {
    type Output = u8;

    fn index(&self, index: Register) -> &Self::Output {
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

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
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

pub enum Register {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
  IndirectHL,
}

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

pub enum Condition {
  Always, Zero, NonZero, Carry, NonCarry
}

pub enum Instruction {
  LD(Operand, Operand),
  XOR(Operand),
  AND(Operand),
  ADD(Register, Operand),
  ADC(Operand),
  SUB(Operand),
  SBC(Operand),
  OR(Operand),
  CP(Operand),
  INC(Register),
  DEC(Register),
  RLC(Register),
  RRC(Register),
  RL(Register),
  RR(Register),
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
  HALT
}

pub enum Operand {
  Immediate8(u8),
  Immediate16(u16),
  Register(Register),
  RegisterPair(RegisterPair)
}

impl CPU {
  fn get_register_pair(&self, rp: RegisterPair) -> u16 {
    match rp {
      RegisterPair::BC => &((self.registers.b as u16) << 8) | self.registers.c as u16,
      RegisterPair::DE => &((self.registers.d as u16) << 8) | self.registers.e as u16,
      RegisterPair::HL => &((self.registers.h as u16) << 8) | self.registers.l as u16,
      RegisterPair::AF => &((self.registers.a as u16) << 8) | if self.flags.z { 0x8 } else { 0 } | if self.flags.n { 0x4 } else { 0 } | if self.flags.h { 0x2 } else { 0 } | if self.flags.c { 0x1 } else { 0 },
      RegisterPair::SP => self.registers.sp,
      _ => panic!("Illegal register pair")
    }
  }

  fn set_register_pair(&mut self, rp: RegisterPair, value: u16) {
    match rp {
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
    let value = self.bus.fetch_byte(self.registers.pc);
    self.registers.pc = self.registers.pc.wrapping_add(1);
    return value;
  }

  fn fetch_imm16(&mut self) -> u16 {
    let value = self.bus.fetch_word(self.registers.pc);
    self.registers.pc = self.registers.pc.wrapping_add(2);
    return value;
  }

  pub fn fetch(&mut self) -> u8 {
    self.fetch_imm8()
  }

  pub fn decode(&mut self, opcode: u8) -> Instruction {
    match opcode {
      0x00 => Instruction::NOP,
      0o20 => Instruction::STOP,
      0o30 => Instruction::JR(Condition::Always, self.fetch_imm8() as i8),
      0o40 | 0o50 | 0o60 | 0o70 => Instruction::JR(inherent_condition_operand(&((opcode - 0o40) >> 3)), self.fetch_imm8() as i8),
      0o04 | 0o14 | 0o24 | 0o34 | 0o44 | 0o54 | 0o64 | 0o74 => Instruction::INC(inherent_register_operand(&((opcode - 0o70) >> 3))),
      0o05 | 0o15 | 0o25 | 0o35 | 0o45 | 0o55 | 0o65 | 0o75 => Instruction::DEC(inherent_register_operand(&((opcode - 0o70) >> 3))),
      0o06 | 0o16 | 0o26 | 0o36 | 0o46 | 0o56 | 0o66 | 0o76 => Instruction::LD(Operand::Register(inherent_register_operand(&((opcode - 0o70) >> 3))), Operand::Immediate8(self.fetch_imm8())),
      0o41 => Instruction::LD(Operand::RegisterPair(RegisterPair::HL), Operand::Immediate16(self.fetch_imm16())),
      0o61 => Instruction::LD(Operand::RegisterPair(RegisterPair::SP), Operand::Immediate16(self.fetch_imm16())),
      0o166 => Instruction::HALT,
      0o100..=0o177 => Instruction::LD(Operand::Register(inherent_register_operand(&((opcode - 0o70) >> 3))), Operand::Register(inherent_register_operand(&opcode))),
      0o200..=0o207 => Instruction::ADD(Register::A, Operand::Register(inherent_register_operand(&opcode))),
      0o210..=0o217 => Instruction::ADC(Operand::Register(inherent_register_operand(&opcode))),
      0o220..=0o227 => Instruction::SUB(Operand::Register(inherent_register_operand(&opcode))),
      0o230..=0o237 => Instruction::SBC(Operand::Register(inherent_register_operand(&opcode))),
      0o240..=0o247 => Instruction::AND(Operand::Register(inherent_register_operand(&opcode))),
      0o250..=0o257 => Instruction::XOR(Operand::Register(inherent_register_operand(&opcode))),
      0o260..=0o267 => Instruction::OR(Operand::Register(inherent_register_operand(&opcode))),
      0o270..=0o277 => Instruction::CP(Operand::Register(inherent_register_operand(&opcode))),
      0o300 | 0o310 | 0o320 | 0o330 => Instruction::RET(inherent_condition_operand(&((opcode - 0o40) >> 3))),
      0o311 => Instruction::RET(Condition::Always),
      0o303 => Instruction::JP(Condition::Always, Operand::Immediate16(self.fetch_imm16())),
      0o351 => Instruction::JP(Condition::Always, Operand::RegisterPair(RegisterPair::HL)),
      0o302 | 0o312 | 0o322 | 0o332 => Instruction::JP(inherent_condition_operand(&((opcode - 0o300) >> 3)), Operand::Immediate16(self.fetch_imm16())),
      0o304 | 0o314 | 0o324 | 0o334 => Instruction::CALL(inherent_condition_operand(&((opcode - 0o300) >> 3)), self.fetch_imm16()),
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
      0o306 => Instruction::ADD(Register::A, Operand::Immediate8(self.fetch_imm8())),
      0o316 => Instruction::ADC(Operand::Immediate8(self.fetch_imm8())),
      0o326 => Instruction::SUB(Operand::Immediate8(self.fetch_imm8())),
      0o336 => Instruction::SBC(Operand::Immediate8(self.fetch_imm8())),
      0o346 => Instruction::AND(Operand::Immediate8(self.fetch_imm8())),
      0o356 => Instruction::XOR(Operand::Immediate8(self.fetch_imm8())),
      0o366 => Instruction::OR(Operand::Immediate8(self.fetch_imm8())),
      0o376 => Instruction::CP(Operand::Immediate8(self.fetch_imm8())),
      0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => Instruction::RST(opcode & 0o70),
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
          (Operand::Register(target), Operand::Register(source)) => self.registers[target] = self.registers[source],
          (Operand::Register(target), Operand::Immediate8(value)) => self.registers[target] = value,
          (Operand::RegisterPair(target), Operand::Immediate16(value)) => self.set_register_pair(target, value),
          _ => panic!("Illegal LD target/source")
        }
      },
      Instruction::XOR(operand) => {
        self.registers.a ^= match operand {
          Operand::Register(register) => self.registers[register],
          Operand::Immediate8(value) => value,
          _ => panic!("Illegal XOR operand")
        };
        self.flags.z = self.registers.a == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
      },
      _ => panic!("Unhandled instruction")
    }
  }
}