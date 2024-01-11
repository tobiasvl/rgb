use rgb_emu::cartridge;
use rgb_emu::cpu::*;

#[test]
fn test_initial_state_bootrom() {
    let mut cpu = Cpu::new();

    let bootrom = std::fs::read("boot.gb").expect("Test requires bootrom");
    cpu.bus.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);
    cpu.bus.bootrom_enabled = true;

    let testrom = std::fs::read("gb-test-roms/cpu_instrs/individual/06-ld r,r.gb")
        .expect("Test requires cartridge");
    cpu.bus.cartridge = Some(cartridge::from_rom(testrom));

    loop {
        println!("PC: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X} ({:02X}{:02X}), ({:02X} {:02X} {:02X} {:02X})",
      cpu.registers.pc,
      cpu.get_register_pair(&RegisterPair::AF),
      cpu.get_register_pair(&RegisterPair::BC),
      cpu.get_register_pair(&RegisterPair::DE),
      cpu.get_register_pair(&RegisterPair::HL),
      cpu.get_register_pair(&RegisterPair::SP),
      cpu.bus.read_byte(cpu.registers.sp),
      cpu.bus.read_byte(cpu.registers.sp+1),
      cpu.bus.read_byte(cpu.registers.pc),
      cpu.bus.read_byte(cpu.registers.pc+1),
      cpu.bus.read_byte(cpu.registers.pc+2),
      cpu.bus.read_byte(cpu.registers.pc+3),
    );
        // TODO check for interrupts
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
        if cpu.registers.pc == 0x100 {
            break;
        };
    }
    assert_eq!(cpu.registers.pc, 0x100);
    assert_eq!(cpu.registers.a, 0x01);
    assert_eq!(cpu.registers.b, 0x00);
    assert_eq!(cpu.registers.c, 0x13);
    assert_eq!(cpu.registers.d, 0x00);
    assert_eq!(cpu.registers.e, 0xD8);
    assert_eq!(cpu.registers.h, 0x01);
    assert_eq!(cpu.registers.l, 0x4D);
    assert!(cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(cpu.flags.h);
    assert!(cpu.flags.c);
    assert_eq!(cpu.registers.sp, 0xFFFE);
}

#[test]
fn test_initial_state_bootrom_no_cart() {
    let mut cpu = Cpu::new();

    let bootrom = std::fs::read("boot.gb").expect("Test requires bootrom");
    cpu.bus.bootrom[0..=0xFF].clone_from_slice(&bootrom[..]);
    cpu.bus.bootrom_enabled = true;

    loop {
        // TODO check for interrupts
        let opcode = cpu.fetch();
        let instruction = cpu.decode(opcode);
        cpu.execute(instruction);
        if cpu.registers.pc == 0xFA {
            break;
        };
    }
    assert_eq!(cpu.registers.pc, 0xFA);
    assert_eq!(cpu.registers.a, 0xFF); // TODO check this
    assert_eq!(cpu.registers.b, 0x00);
    assert_eq!(cpu.registers.c, 0x13);
    assert_eq!(cpu.registers.d, 0x00);
    assert_eq!(cpu.registers.e, 0xD8);
    assert_eq!(cpu.registers.h, 0x01);
    assert_eq!(cpu.registers.l, 0x4D);
    assert!(!cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h); // TODO check this
    assert!(!cpu.flags.c);
    assert_eq!(cpu.registers.sp, 0xFFFE);
}
