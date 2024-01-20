use pretty_assertions::assert_eq;
use std::collections::HashMap;

use rgb_emu::bus::Bus;
use rgb_emu::cartridge::Cartridge;
use rgb_emu::cpu::*;
use serde::{Deserialize, Serialize};

struct JsMooBus {
    pub ram: HashMap<u16, u8>,
    pub interrupt_enable: u8,
}

impl JsMooBus {
    pub fn new() -> Self {
        Self {
            ram: HashMap::<u16, u8>::new(), //[0; 0x10000],
            interrupt_enable: 0,
        }
    }
}

impl Bus for JsMooBus {
    fn tick(&mut self) {}
    fn peek_byte(&self, address: u16) -> u8 {
        *(self.ram.get(&address).unwrap_or(&0u8))
    }
    fn read_byte(&mut self, address: u16) -> u8 {
        self.peek_byte(address)
    }
    fn read_word(&mut self, address: u16) -> u16 {
        let low_byte = u16::from(self.read_byte(address));
        u16::from(self.read_byte(address + 1)) << 8 | low_byte
    }
    fn write_byte(&mut self, address: u16, value: u8) {
        self.ram.insert(address, value);
    }
    fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
    }
    fn set_post_boot_state(&mut self) {}
    fn set_interrupt_enable(&mut self, value: u8) {
        self.interrupt_enable = value;
    }
    fn get_interrupt_enable(&self) -> u8 {
        self.interrupt_enable
    }
    fn get_interrupt_flags(&self) -> u8 {
        0
    }
    fn set_interrupt_flags(&mut self, _: u8) {}

    fn insert_cartridge(&mut self, _: Box<dyn Cartridge>) {}
    fn remove_cartridge(&mut self) {}
    fn set_boot_rom(&mut self, _: Vec<u8>) {}
}

#[derive(Serialize, Deserialize, Debug)]
struct CpuTest {
    name: String,
    #[serde(rename = "initial")]
    initial_state: CpuState,
    #[serde(rename = "final")]
    final_state: CpuState,
}

#[derive(Serialize, Deserialize, Debug)]
struct CpuState {
    pc: u16,
    sp: u16,
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    ime: u8,
    #[serde(default)]
    ie: u8,
    #[serde(default)]
    ei: u8, // ime_delayed
    ram: Vec<[u16; 2]>,
}

impl PartialEq for CpuState {
    fn eq(&self, other: &CpuState) -> bool {
        for [ram_address, value] in &self.ram {
            if value != &0 && !other.ram.contains(&[*ram_address, *value]) {
                return false;
            }
        }

        self.pc == other.pc
            && self.sp == other.sp
            && self.a == other.a
            && self.b == other.b
            && self.c == other.c
            && self.d == other.d
            && self.e == other.e
            && self.h == other.h
            && self.l == other.l
            && self.ime == other.ime
            && self.ei == other.ei
            && self.f == other.f
    }
}

impl From<&Cpu> for CpuState {
    fn from(cpu: &Cpu) -> CpuState {
        CpuState {
            pc: cpu.registers.pc,
            sp: cpu.registers.sp,
            a: cpu.registers.a,
            f: (u8::from(cpu.flags.c) << 4)
                | (u8::from(cpu.flags.h) << 5)
                | (u8::from(cpu.flags.n) << 6)
                | (u8::from(cpu.flags.z) << 7),
            b: cpu.registers.b,
            c: cpu.registers.c,
            d: cpu.registers.d,
            e: cpu.registers.e,
            h: cpu.registers.h,
            l: cpu.registers.l,
            ime: u8::from(cpu.ime),
            ie: cpu.bus.get_interrupt_enable(),
            ei: u8::from(cpu.ime_delayed),
            ram: {
                let mut ram = Vec::<[u16; 2]>::new();
                for address in 0x0000..=0xFFFF {
                    let value = cpu.bus.peek_byte(address);
                    if value != 0 {
                        ram.push([address, u16::from(value)]);
                    }
                }
                ram
            },
        }
    }
}

impl From<CpuState> for Cpu {
    fn from(cpu_state: CpuState) -> Cpu {
        #![allow(clippy::cast_possible_truncation)]
        let mut cpu = Cpu {
            registers: Registers {
                pc: cpu_state.pc,
                sp: cpu_state.sp,
                a: cpu_state.a,
                b: cpu_state.b,
                c: cpu_state.c,
                d: cpu_state.d,
                e: cpu_state.e,
                h: cpu_state.h,
                l: cpu_state.l,
            },
            flags: Flags {
                c: ((cpu_state.f >> 4) & 1) == 1,
                h: ((cpu_state.f >> 5) & 1) == 1,
                n: ((cpu_state.f >> 6) & 1) == 1,
                z: ((cpu_state.f >> 7) & 1) == 1,
            },
            ime: cpu_state.ime == 1,
            ime_delayed: cpu_state.ei == 1,
            bus: Box::new(JsMooBus::new()),
            ..Cpu::default()
        };

        for [ram_address, value] in cpu_state.ram {
            cpu.bus.write_byte(ram_address, value as u8);
        }

        cpu
    }
}

fn set_state_from(cpu: &mut Cpu, cpu_state: CpuState) {
    #![allow(clippy::cast_possible_truncation)]
    cpu.registers.pc = cpu_state.pc;
    cpu.registers.sp = cpu_state.sp;
    cpu.registers.a = cpu_state.a;
    cpu.registers.b = cpu_state.b;
    cpu.registers.c = cpu_state.c;
    cpu.registers.d = cpu_state.d;
    cpu.registers.e = cpu_state.e;
    cpu.registers.h = cpu_state.h;
    cpu.registers.l = cpu_state.l;
    cpu.flags.c = ((cpu_state.f >> 4) & 1) == 1;
    cpu.flags.h = ((cpu_state.f >> 5) & 1) == 1;
    cpu.flags.n = ((cpu_state.f >> 6) & 1) == 1;
    cpu.flags.z = ((cpu_state.f >> 7) & 1) == 1;
    cpu.ime = cpu_state.ime == 1;
    cpu.ime_delayed = cpu_state.ei == 1;

    cpu.bus = Box::new(JsMooBus::new());
    for [ram_address, value] in cpu_state.ram {
        cpu.bus.write_byte(ram_address, value as u8);
    }
}

#[test]
pub(crate) fn jsmoo() -> Result<(), String> {
    #![allow(clippy::unwrap_used)]
    let mut cpu = Cpu::new(); // TODO fix bus
    for i in [0x00, 0xCB] {
        for j in 0x00..=0xFF {
            let opcode = (i << 8) | j;
            let skip_opcodes = [
                0x0027, // TODO STOP
                0x00CB, // Prefix opcode
                // Illegal opcodes:
                0x00D3, 0x00DB, 0x00DD, 0x00E3, 0x00E4, 0x00EB, 0x00EC, 0x00ED, 0x00F4, 0x00FC,
                0x00FD,
            ];

            if skip_opcodes.contains(&opcode) {
                continue;
            }

            println!("Testing opcode {opcode:02x}...");

            let filename = if opcode > 0x00FF {
                format!("cb {:02x}", opcode & 0xFF)
            } else {
                format!("{opcode:02x}")
            };
            let tests = serde_json::from_str::<Vec<CpuTest>>(
                &String::from_utf8(
                    std::fs::read(format!(
                        "tests/jsmoo-sm83-tests/misc/tests/GeneratedTests/sm83/v1/{filename}.json",
                    ))
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();

            for test in tests {
                set_state_from(&mut cpu, test.initial_state);

                let opcode = cpu.fetch();
                let opcode = cpu.decode(opcode);
                let opcode_name = format!("{opcode:?}");
                cpu.execute(opcode);

                let final_state = CpuState::from(&cpu);
                if final_state != test.final_state {
                    assert_eq!(final_state, test.final_state);
                    return Err(format!("{} ({opcode_name})", test.name));
                }
            }
        }
    }

    Ok(())
}
