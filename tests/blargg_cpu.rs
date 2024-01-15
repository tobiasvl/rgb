mod blargg;
use blargg::run_blargg_test;

/// This is a special test ROM containing all the `blargg_cpu_*` tests.
/// It's a separate test because it uses a different MBC and therefore tests a bit more.
#[test]
fn blargg_cpu_all() -> Result<(), String> {
    run_blargg_test("cpu_instrs/cpu_instrs.gb")
}

#[test]
fn blargg_cpu_01() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/01-special.gb")
}

#[test]
fn blargg_cpu_02() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/02-interrupts.gb")
}

#[test]
fn blargg_cpu_03() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/03-op sp,hl.gb")
}

#[test]
fn blargg_cpu_04() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/04-op r,imm.gb")
}

#[test]
fn blargg_cpu_05() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/05-op rp.gb")
}

#[test]
fn blargg_cpu_06() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/06-ld r,r.gb")
}

#[test]
fn blargg_cpu_07() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/07-jr,jp,call,ret,rst.gb")
}

#[test]
fn blargg_cpu_08() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/08-misc instrs.gb")
}

#[test]
fn blargg_cpu_09() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/09-op r,r.gb")
}

#[test]
fn blargg_cpu_10() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/10-bit ops.gb")
}

#[test]
fn blargg_cpu_11() -> Result<(), String> {
    run_blargg_test("cpu_instrs/individual/11-op a,(hl).gb")
}
