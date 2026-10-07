#![allow(dead_code)]
use core::arch::asm;

fn current_mode() -> u32 {
  unsafe {
    let mode: u32;
    asm!("mrs {}, cpsr", out(reg) mode);
    mode & 0x1f
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn irq_handler() {
  panic!("holy shit we triggered irq");
}

pub fn enable() {
  unsafe {
    asm!(
        "ldr r1, =_Reset",
        "mcr p15, 0, r1, c12, c0, 0",
        "isb",
        "cpsie i",
        out("r1") _,
    );
  }
}
