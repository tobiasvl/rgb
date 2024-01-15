#![allow(clippy::unwrap_used)]
use rgb_emu::cartridge;
use rgb_emu::cpu::*;

pub(crate) fn run_blargg_test(path: &str) -> Result<(), String> {
    let mut cpu = Cpu::new();
    cpu.set_post_boot_state();

    let rom =
        std::fs::read(String::from("tests/gb-test-roms/") + path).expect("Unable to open ROM");

    cpu.bus.cartridge = Some(cartridge::from_rom(rom));

    let mut serial_output: String = String::new();

    loop {
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
        if cpu.bus.read_byte(0xFF02) != 0 {
            let character = cpu.bus.read_byte(0xFF01) as char;
            if character == '\n' {
                if serial_output.ends_with("Passed") {
                    return Ok(());
                } else if serial_output.lines().last().unwrap().starts_with("Failed") {
                    return Err(serial_output);
                }
            }
            serial_output.push(character);
            cpu.bus.write_byte(0xFF02, 0);
        }
    }
}
