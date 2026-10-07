#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::kprint::k_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! kprintln {
    () => {
        k_print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::kprint::k_print(format_args!("{}\n", format_args!($($arg)*)))
    };
}

pub struct KernelWriter(i32);

impl KernelWriter {
  fn new() -> Self {
    Self(0x1c090000)
  }
}

impl core::fmt::Write for KernelWriter {
  fn write_str(&mut self, text: &str) -> core::fmt::Result {
    for c in text.chars() {
      unsafe { *((self.0) as *mut u8) = c as u8 }
    }
    Ok(())
  }
}

pub fn k_print(args: core::fmt::Arguments) {
  core::fmt::write(&mut KernelWriter::new(), args).unwrap();
}
