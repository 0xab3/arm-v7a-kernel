pub struct BytesWriter<'a> {
  pub buf: &'a mut [u8],
  pub cursor: usize,
}

impl<'a> BytesWriter<'a> {
  pub fn new(buf: &'a mut [u8]) -> Self {
    Self { buf, cursor: 0 }
  }

  pub fn write_u8(&mut self, data: u8) {
    self.buf[self.cursor] = data;
    self.cursor += 1;
  }
  pub fn write_u16(&mut self, data: u16) {
    self.buf[self.cursor..self.cursor + 2].copy_from_slice(&data.to_be_bytes());
    self.cursor += 2;
  }
  pub fn write_u32(&mut self, data: u32) {
    self.buf[self.cursor..self.cursor + 4].copy_from_slice(&data.to_be_bytes());
    self.cursor += 4;
  }

  pub fn write_bytes(&mut self, data: &[u8]) {
    self.buf[self.cursor..self.cursor + data.len()].copy_from_slice(&data);
    self.cursor += data.len();
  }
}
