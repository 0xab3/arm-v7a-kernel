use core::arch::asm;

use crate::kprintln;

pub fn is_supported() -> bool {
  unsafe {
    let mode: u32;
    asm!("mrc p15, 0, {}, c0, c1, 4", out(reg) mode);
    kprintln!("mod: {:04b}", mode & 0xf);
    false
  }
}
